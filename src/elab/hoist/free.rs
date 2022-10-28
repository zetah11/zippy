use std::collections::HashMap;

use im::HashSet;

use crate::message::Span;
use crate::mir::{BranchNode, Decls, ExprNode, ExprSeq, ValueNode};
use crate::resolve::names::Name;

pub fn free_vars(decls: &Decls) -> HashMap<Name, Vec<(Name, Span)>> {
    let global = decls.defs.iter().map(|def| def.name).collect();
    let mut freer = Freer::new(global);
    freer.calculate_free(decls);

    freer.funs
}

#[derive(Debug)]
struct Freer {
    funs: HashMap<Name, Vec<(Name, Span)>>,
    global: HashSet<Name>,
}

impl Freer {
    pub fn new(global: HashSet<Name>) -> Self {
        Self {
            funs: HashMap::new(),
            global,
        }
    }

    pub fn calculate_free(&mut self, decls: &Decls) {
        for def in decls.defs.iter() {
            let res = self.free_in_function(&[], &def.bind);
            if !res.is_empty() {
                self.funs.insert(def.name, res);
            }
        }
    }

    fn free_in_function(&mut self, params: &[Name], body: &ExprSeq) -> Vec<(Name, Span)> {
        let mut res = Vec::new();
        let mut bound = self.global.clone();
        let mut free = HashSet::new();

        bound.extend(params.iter().copied());

        for expr in body.exprs.iter() {
            match &expr.node {
                ExprNode::Join { name, param, body } => {
                    bound.insert(*name);

                    let free_here = self.free_in_function(&[*param], body);

                    for (name, span) in free_here {
                        if !bound.contains(&name) && free.insert(name).is_none() {
                            res.push((name, span));
                        }
                    }
                }

                ExprNode::Function { name, params, body } => {
                    let free_here = self.free_in_function(params, body);

                    for (name, span) in free_here.iter().copied() {
                        if !bound.contains(&name) && free.insert(name).is_none() {
                            res.push((name, span));
                        }
                    }

                    if !free_here.is_empty() {
                        self.funs.insert(*name, free_here);
                    }

                    bound.insert(*name);
                }

                ExprNode::Apply { names, fun, args } => {
                    if !bound.contains(fun) && free.insert(*fun).is_none() {
                        res.push((*fun, expr.span));
                    }

                    for arg in args.iter() {
                        if let ValueNode::Name(name) = arg.node {
                            if !bound.contains(&name) && free.insert(name).is_none() {
                                res.push((name, arg.span));
                            }
                        }
                    }

                    bound.extend(names.iter().copied());
                }

                ExprNode::Tuple { name, values } => {
                    for value in values.iter() {
                        if let ValueNode::Name(name) = value.node {
                            if !bound.contains(&name) && free.insert(name).is_none() {
                                res.push((name, value.span));
                            }
                        }
                    }

                    bound.insert(*name);
                }

                ExprNode::Proj { name, of, at: _ } => {
                    if !bound.contains(of) && free.insert(*of).is_none() {
                        res.push((*of, expr.span));
                    }

                    bound.insert(*name);
                }
            }
        }

        match &body.branch.node {
            BranchNode::Return(values) => {
                for value in values.iter() {
                    if let ValueNode::Name(name) = value.node {
                        if !bound.contains(&name) && free.insert(name).is_none() {
                            res.push((name, value.span));
                        }
                    }
                }
            }

            BranchNode::Jump(to, arg) => {
                if let ValueNode::Name(name) = arg.node {
                    if !bound.contains(&name) && free.insert(name).is_none() {
                        res.push((name, arg.span));
                    }
                }

                if !bound.contains(to) {
                    unreachable!() // bad layout!
                }
            }
        }

        res
    }
}
