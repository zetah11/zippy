use std::collections::HashMap;

use log::{debug, trace};

use crate::message::{Messages, Span};
use crate::mir::{BranchNode, Context, Decls, Expr, ExprNode, ExprSeq, Value, ValueNode};
use crate::resolve::names::{Name, Names};
use crate::Driver;

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

    functions: HashMap<Name, (Name, ExprSeq)>,
    values: HashMap<Name, Value>,
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
            let mut init = Vec::new();
            let bind_span = def.bind.span;
            let bind_ty = def.bind.ty;
            self.hoist_value(def.name, &mut init, &free_vars, def.bind);

            if !init.is_empty() {
                messages.at(bind_span).elab_requires_init();
                self.values.insert(
                    def.name,
                    Value {
                        node: ValueNode::Invalid,
                        span: bind_span,
                        ty: bind_ty,
                    },
                );
            }
        }

        self.driver.report(messages);
    }

    fn hoist_value(
        &mut self,
        name_for: Name,
        within: &mut Vec<Expr>,
        free_vars: &HashMap<Name, Vec<(Name, Span)>>,
        exprs: ExprSeq,
    ) {
        for expr in exprs.exprs {
            match expr.node {
                ExprNode::Function { name, param, body } => {
                    let body = self.hoist_function(free_vars, body);
                    self.functions.insert(name, (param, body));
                }

                ExprNode::Join { .. } => todo!(),

                node => {
                    within.push(Expr {
                        node,
                        span: expr.span,
                        ty: expr.ty,
                    });
                }
            }
        }

        let value = match exprs.branch.node {
            BranchNode::Return(value) => match value.node {
                ValueNode::Int(_) | ValueNode::Invalid => value,
                ValueNode::Name(name) => {
                    if let Some(function) = self.functions.get(&name) {
                        self.functions.insert(name_for, function.clone());
                        return;
                    } else if let Some(value) = self.values.get(&name) {
                        value.clone()
                    } else {
                        todo!()
                    }
                }
            },

            BranchNode::Jump(..) => todo!(),
        };

        self.values.insert(name_for, value);
    }

    fn hoist_function(
        &mut self,
        free_vars: &HashMap<Name, Vec<(Name, Span)>>,
        exprs: ExprSeq,
    ) -> ExprSeq {
        let mut res = Vec::with_capacity(exprs.exprs.len());

        for expr in exprs.exprs {
            match expr.node {
                ExprNode::Function { name, param, body } => {
                    let invalid = free_vars
                        .get(&name)
                        .map(|free| !free.is_empty())
                        .unwrap_or(false);

                    if invalid {
                        let bind = Value {
                            node: ValueNode::Invalid,
                            span: expr.span,
                            ty: expr.ty,
                        };

                        self.values.insert(name, bind);
                        continue;
                    }

                    let body = self.hoist_function(free_vars, body);
                    self.functions.insert(name, (param, body));
                }

                ExprNode::Join { .. } => todo!(),

                node => res.push(Expr {
                    node,
                    span: expr.span,
                    ty: expr.ty,
                }),
            }
        }

        res.shrink_to_fit();

        ExprSeq::new(exprs.span, exprs.ty, res, exprs.branch)
    }
}
