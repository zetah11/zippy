use std::collections::HashMap;

use crate::message::{Messages, Span};
use crate::mir::{
    Branch, BranchNode, Context, Decls, Expr, ExprNode, ExprSeq, Value, ValueDef, ValueNode,
};
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
    let mut hoister = Hoist {
        driver,
        names,
        context,
    };
    hoister.hoist_decls(decls)
}

pub struct Hoist<'a, D> {
    driver: &'a mut D,
    names: &'a mut Names,
    context: &'a mut Context,
}

impl<D: Driver> Hoist<'_, D> {
    fn hoist_decls(&mut self, decls: Decls) -> Decls {
        let free_vars = free_vars(&decls);
        let mut messages = Messages::new();

        for (_, free) in free_vars.iter() {
            if !free.is_empty() {
                messages.elab_closure_not_permitted(free.iter().map(|(_, span)| *span))
            }
        }

        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values {
            let mut res = Vec::with_capacity(def.bind.exprs.len());

            for expr in def.bind.exprs {
                if let ExprNode::Function { name, param, body } = expr.node {
                    let body = self.hoist_exprs(&free_vars, &mut values, body);
                    res.push(Expr {
                        node: ExprNode::Function { name, param, body },
                        span: expr.span,
                        ty: expr.ty,
                    });

                    continue;
                }

                res.push(expr);
            }

            let bind = ExprSeq::new(def.bind.span, def.bind.ty, res, def.bind.branch);
            values.push(ValueDef {
                name: def.name,
                span: def.span,
                bind,
            });
        }

        self.driver.report(messages);

        Decls { values }
    }

    fn hoist_exprs(
        &mut self,
        free_vars: &HashMap<Name, Vec<(Name, Span)>>,
        values: &mut Vec<ValueDef>,
        exprs: ExprSeq,
    ) -> ExprSeq {
        let mut res = Vec::with_capacity(exprs.exprs.len());

        for expr in exprs.exprs {
            if let ExprNode::Function { name, param, body } = expr.node {
                let invalid = free_vars
                    .get(&name)
                    .map(|free| !free.is_empty())
                    .unwrap_or(false);

                if invalid {
                    let bind = ExprSeq::new(
                        expr.span,
                        expr.ty,
                        vec![],
                        Branch {
                            node: BranchNode::Return(Value {
                                node: ValueNode::Invalid,
                                span: expr.span,
                                ty: expr.ty,
                            }),
                            span: expr.span,
                            ty: expr.ty,
                        },
                    );

                    values.push(ValueDef {
                        name,
                        bind,
                        span: expr.span,
                    });

                    continue;
                }

                let Expr { span, ty, .. } = expr;

                let body = self.hoist_exprs(free_vars, values, body);

                let new_name = self.names.fresh(span, None);
                self.context.add(new_name, ty);

                let expr = Expr {
                    node: ExprNode::Function {
                        name: new_name,
                        param,
                        body,
                    },
                    span,
                    ty,
                };

                let bind = ExprSeq {
                    exprs: vec![expr],
                    branch: Branch {
                        node: BranchNode::Return(Value {
                            node: ValueNode::Name(new_name),
                            span,
                            ty,
                        }),
                        span,
                        ty,
                    },
                    span,
                    ty,
                };

                values.push(ValueDef { name, bind, span });

                continue;
            }

            res.push(expr);
        }

        res.shrink_to_fit();
        ExprSeq {
            exprs: res,
            branch: exprs.branch,
            span: exprs.span,
            ty: exprs.ty,
        }
    }
}
