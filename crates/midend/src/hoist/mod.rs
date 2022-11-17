use std::collections::HashMap;

use log::{debug, trace};

use common::message::{Messages, Span};
use common::mir::{
    Block, Branch, BranchNode, Context, Decls, Statement, StaticValue, StaticValueNode, StmtNode,
    Value, ValueNode,
};
use common::names::{Name, Names};
use common::Driver;

mod free;

use free::free_vars;

pub fn hoist(
    driver: &mut impl Driver,
    names: &mut Names,
    context: &mut Context,
    decls: Decls,
) -> Decls {
    debug!("beginning hoisting");

    let mut hoister = Hoist {
        driver,
        _names: names,
        _context: context,

        functions: HashMap::new(),
        values: HashMap::new(),
    };

    hoister.hoist_decls(decls);
    let res = Decls {
        defs: Vec::new(),
        functions: hoister.functions,
        values: hoister.values,
    };

    trace!("done hoisting");

    res
}

pub struct Hoist<'a, D> {
    driver: &'a mut D,
    _names: &'a mut Names,
    _context: &'a mut Context,

    functions: HashMap<Name, (Vec<Name>, Block)>,
    values: HashMap<Name, StaticValue>,
}

impl<D: Driver> Hoist<'_, D> {
    fn hoist_decls(&mut self, decls: Decls) {
        let free_vars = free_vars(&decls);
        let mut messages = Messages::new();

        for (_, free) in free_vars.iter() {
            if !free.is_empty() {
                messages.elab_closure_not_permitted(free.iter().map(|(_, span)| *span))
            }
        }

        for def in decls.defs {
            self.hoist_value(def.name, &free_vars, def.bind);
        }

        self.driver.report(messages);
    }

    fn hoist_value(
        &mut self,
        name_for: Name,
        free_vars: &HashMap<Name, Vec<(Name, Span)>>,
        exprs: Block,
    ) {
        let mut init = Vec::with_capacity(exprs.exprs.len());

        for expr in exprs.exprs {
            match expr.node {
                StmtNode::Function { name, params, body } => {
                    let body = self.hoist_function(free_vars, body);
                    self.functions.insert(name, (params, body));
                }

                StmtNode::Join { .. } => todo!(),

                node => {
                    init.push(Statement {
                        node,
                        span: expr.span,
                        ty: expr.ty,
                    });
                }
            }
        }

        let value = match exprs.branch.node {
            BranchNode::Jump(..) => todo!(),
            BranchNode::Return(mut values) => {
                if values.len() != 1 {
                    // tuple shenanigans?
                    todo!()
                } else {
                    let value = values.remove(0);
                    let span = value.span;
                    let ty = value.ty;
                    let node = match value.node {
                        ValueNode::Int(i) => StaticValueNode::Int(i),
                        ValueNode::Invalid => {
                            let node = BranchNode::Return(vec![value]);
                            let branch = Branch { node, span, ty };
                            StaticValueNode::LateInit(Block {
                                exprs: init,
                                branch,
                                span,
                                ty,
                            })
                        }
                        ValueNode::Name(name) => {
                            if let Some(function) = self.functions.get(&name) {
                                self.functions.insert(name_for, function.clone());
                                return;
                            } else if let Some(value) = self.values.get(&name) {
                                self.values.insert(name_for, value.clone());
                                return;
                            } else {
                                let node = BranchNode::Return(vec![value]);
                                let branch = Branch { node, span, ty };
                                StaticValueNode::LateInit(Block {
                                    exprs: init,
                                    branch,
                                    span,
                                    ty,
                                })
                            }
                        }
                    };

                    StaticValue { node, span, ty }
                }
            }
        };

        self.values.insert(name_for, value);
    }

    fn hoist_function(
        &mut self,
        free_vars: &HashMap<Name, Vec<(Name, Span)>>,
        exprs: Block,
    ) -> Block {
        let mut res = Vec::with_capacity(exprs.exprs.len());

        for expr in exprs.exprs {
            match expr.node {
                StmtNode::Function { name, params, body } => {
                    let invalid = free_vars
                        .get(&name)
                        .map(|free| !free.is_empty())
                        .unwrap_or(false);

                    if invalid {
                        let span = expr.span;
                        let ty = expr.ty;
                        let value = Value {
                            node: ValueNode::Invalid,
                            span,
                            ty,
                        };

                        let node = BranchNode::Return(vec![value]);
                        let branch = Branch { node, span, ty };

                        let node = StaticValueNode::LateInit(Block {
                            exprs: vec![],
                            branch,
                            span,
                            ty,
                        });

                        let bind = StaticValue { node, span, ty };

                        self.values.insert(name, bind);
                        continue;
                    }

                    let body = self.hoist_function(free_vars, body);
                    self.functions.insert(name, (params, body));
                }

                StmtNode::Join { .. } => todo!(),

                node => res.push(Statement {
                    node,
                    span: expr.span,
                    ty: expr.ty,
                }),
            }
        }

        res.shrink_to_fit();

        Block::new(exprs.span, exprs.ty, res, exprs.branch)
    }
}
