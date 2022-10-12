use im::HashSet;

use super::free::free_in_function;
use crate::mir::{Context, Decls, Expr, ExprNode, ExprSeq, Types, Value, ValueDef, ValueNode};
use crate::resolve::names::{Name, Names};

pub struct Converter<'a> {
    names: &'a mut Names,
    _context: &'a mut Context,
    _types: &'a mut Types,

    non_closures: HashSet<Name>,
}

impl<'a> Converter<'a> {
    pub fn new(names: &'a mut Names, context: &'a mut Context, types: &'a mut Types) -> Self {
        Self {
            names,
            _context: context,
            _types: types,

            non_closures: HashSet::new(),
        }
    }

    pub fn convert(&mut self, decls: Decls) -> Decls {
        let mut values = Vec::with_capacity(decls.values.len());

        let bound: HashSet<_> = decls.values.iter().map(|def| def.name).collect();
        self.non_closures.extend(bound.iter().copied());

        for def in decls.values {
            let bind = self.convert_exprs(&bound, def.bind);
            values.push(ValueDef {
                bind,
                name: def.name,
                span: def.span,
            });
        }

        Decls { values }
    }

    fn convert_exprs(&mut self, bound: &HashSet<Name>, exprs: ExprSeq) -> ExprSeq {
        let mut res = Vec::new();

        for expr in exprs.exprs {
            self.convert_expr(bound, &mut res, expr);
        }

        ExprSeq::new(exprs.span, exprs.ty, res, exprs.branch)
    }

    fn convert_expr(&mut self, bound: &HashSet<Name>, within: &mut Vec<Expr>, expr: Expr) {
        let node = match expr.node {
            ExprNode::Apply { name, fun, arg } => {
                let fun_ptr = self.names.fresh(expr.span, None);
                let closure_arg = self.names.fresh(expr.span, None);
                within.extend([
                    Expr {
                        node: ExprNode::Proj {
                            name: fun_ptr,
                            of: fun,
                            at: 0,
                        },
                        span: expr.span,
                        ty: expr.ty, // todo!
                    },
                    Expr {
                        node: ExprNode::Tuple {
                            name: closure_arg,
                            values: vec![
                                Value {
                                    node: ValueNode::Name(fun),
                                    span: expr.span,
                                    ty: expr.ty, // todo!
                                },
                                arg,
                            ],
                        },
                        span: expr.span,
                        ty: expr.ty,
                    },
                    Expr {
                        node: ExprNode::Apply {
                            name,
                            fun: fun_ptr,
                            arg: Value {
                                node: ValueNode::Name(closure_arg),
                                span: expr.span,
                                ty: expr.ty, // todo!
                            },
                        },
                        span: expr.span,
                        ty: expr.ty,
                    },
                ]);

                return;
            }

            ExprNode::Function { name, param, body } => {
                let free_vars: Vec<_> = free_in_function(&bound.update(param), &param, &body)
                    .into_iter()
                    .collect();

                let env = self.names.fresh(expr.span, None);
                let new_param = self.names.fresh(expr.span, None);

                let mut new_body = Vec::new();

                new_body.extend([
                    Expr {
                        node: ExprNode::Proj {
                            name: env,
                            of: new_param,
                            at: 0,
                        },
                        span: expr.span,
                        ty: expr.ty, // todo!
                    },
                    Expr {
                        node: ExprNode::Proj {
                            name: param,
                            of: new_param,
                            at: 1,
                        },
                        span: expr.span,
                        ty: expr.ty, // todo!
                    },
                ]);

                for (i, free) in free_vars.iter().copied().enumerate() {
                    new_body.push(Expr {
                        node: ExprNode::Proj {
                            name: free,
                            of: env,
                            at: i + 1, // first parameter is fn pointer itself
                        },
                        span: expr.span,
                        ty: expr.ty, // todo!
                    });
                }

                let body = self.convert_exprs(bound, body);
                new_body.extend(body.exprs);

                let new_name = self.names.fresh(expr.span, None);

                within.push(Expr {
                    node: ExprNode::Function {
                        name: new_name,
                        param: new_param,
                        body: ExprSeq::new(body.span, body.ty, new_body, body.branch),
                    },
                    span: expr.span,
                    ty: expr.ty, // todo!
                });

                let values = std::iter::once(Value {
                    node: ValueNode::Name(new_name),
                    span: expr.span,
                    ty: expr.ty, // todo!
                })
                .chain(free_vars.into_iter().map(|var| Value {
                    node: ValueNode::Name(var),
                    span: expr.span,
                    ty: expr.ty,
                }))
                .collect();

                within.push(Expr {
                    node: ExprNode::Tuple { name, values },
                    span: expr.span,
                    ty: expr.ty,
                });

                return;
            }

            ExprNode::Join { name, param, body } => {
                let body = self.convert_exprs(bound, body);
                ExprNode::Join { name, param, body }
            }

            ExprNode::Tuple { name, values } => ExprNode::Tuple { name, values },
            ExprNode::Proj { name, of, at } => ExprNode::Proj { name, of, at },
        };

        within.push(Expr {
            node,
            span: expr.span,
            ty: expr.ty,
        });
    }
}
