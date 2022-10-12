use im::HashSet;

use crate::mir::{ExprNode, ExprSeq, ValueNode};
use crate::resolve::names::Name;

pub fn free_in_function(bound: &HashSet<Name>, param: &Name, body: &ExprSeq) -> HashSet<Name> {
    let mut bound = bound.update(*param);
    let mut res = HashSet::new();

    for expr in body.exprs.iter() {
        match &expr.node {
            ExprNode::Produce(value) => {
                if let ValueNode::Name(name) = &value.node {
                    if !bound.contains(name) {
                        res.insert(*name);
                    }
                }
            }

            ExprNode::Jump(target, value) => {
                assert!(bound.contains(target));

                if let ValueNode::Name(name) = &value.node {
                    if !bound.contains(name) {
                        res.insert(*name);
                    }
                }
            }

            ExprNode::Join { name, param, body } => {
                bound.insert(*name);

                let free_body = free_in_function(&bound, param, body);
                res.extend(free_body.relative_complement(bound.clone()))
            }

            ExprNode::Function { name, param, body } => {
                bound.insert(*name);

                let free_body = free_in_function(&bound, param, body);
                res.extend(free_body.relative_complement(bound.clone()));
            }

            ExprNode::Apply { name, fun, arg } => {
                if !bound.contains(fun) {
                    res.insert(*fun);
                }

                if let ValueNode::Name(name) = &arg.node {
                    if !bound.contains(name) {
                        res.insert(*name);
                    }
                }

                bound.insert(*name);
            }

            ExprNode::Tuple { name, values } => {
                for value in values.iter() {
                    if let ValueNode::Name(name) = &value.node {
                        if !bound.contains(name) {
                            res.insert(*name);
                        }
                    }
                }

                bound.insert(*name);
            }

            ExprNode::Proj { name, of, .. } => {
                if !bound.contains(of) {
                    res.insert(*of);
                }

                bound.insert(*name);
            }
        }
    }

    res
}
