use crate::mir::{Expr, ExprNode, ExprSeq, Type, Value, ValueNode};
use crate::Driver;

use super::{Env, Irreducible, IrreducibleNode, Lowerer};

impl<D: Driver> Lowerer<'_, D> {
    pub fn reduce_exprs(&mut self, env: Env, exprs: ExprSeq) -> Irreducible {
        let mut env = env.child();

        let mut new_exprs = ExprSeq::new(exprs.span, exprs.ty);

        for expr in exprs.exprs {
            let node = match expr.node {
                ExprNode::Produce(Value {
                    node: ValueNode::Int(i),
                    ..
                }) => {
                    return Irreducible {
                        node: IrreducibleNode::Integer(i),
                        span: expr.span,
                        ty: expr.ty,
                    };
                }
                ExprNode::Produce(Value {
                    node: ValueNode::Invalid,
                    ..
                }) => {
                    return Irreducible {
                        node: IrreducibleNode::Invalid,
                        span: expr.span,
                        ty: expr.ty,
                    };
                }
                ExprNode::Produce(Value {
                    node: ValueNode::Name(name),
                    span,
                    ty,
                }) => {
                    if let Some(value) = env.lookup(&name) {
                        return value.clone();
                    } else {
                        ExprNode::Produce(Value {
                            node: ValueNode::Name(name),
                            span,
                            ty,
                        })
                    }
                }

                ExprNode::Jump(..) | ExprNode::Join { .. } => todo!(),

                ExprNode::Function { name, param, body } => {
                    let body_irr = self.reduce_exprs(env.clone(), body.clone());
                    env.set(
                        name,
                        Irreducible {
                            node: IrreducibleNode::Lambda(param, Box::new(body_irr), env.clone()),
                            span: expr.span,
                            ty: expr.ty,
                        },
                    );

                    ExprNode::Function { name, param, body }
                }

                ExprNode::Apply { name, fun, arg } => {
                    if let Some(Irreducible {
                        node: IrreducibleNode::Lambda(param, body, closed),
                        ..
                    }) = env.lookup(&fun)
                    {
                        let arg = self.reduce_value(&env, arg);
                        let result = self.reduce_irr(closed.with(*param, arg), *body.clone());

                        env.set(name, result);

                        continue;
                    } else {
                        ExprNode::Apply { name, fun, arg }
                    }
                }

                ExprNode::Tuple { name, values } => {
                    let values = values
                        .into_iter()
                        .map(|value| self.reduce_value(&env, value))
                        .collect();
                    env.set(
                        name,
                        Irreducible {
                            node: IrreducibleNode::Tuple(values),
                            span: expr.span,
                            ty: expr.ty,
                        },
                    );
                    continue;
                }

                ExprNode::Proj { name, of, at } => {
                    if let Some(Irreducible {
                        node: IrreducibleNode::Tuple(values),
                        ..
                    }) = env.lookup(&of)
                    {
                        env.set(name, values[at].clone());
                        continue;
                    } else {
                        ExprNode::Proj { name, of, at }
                    }
                }
            };

            new_exprs.push(Expr {
                node,
                span: expr.span,
                ty: expr.ty,
            })
        }

        Irreducible {
            node: IrreducibleNode::Quote(new_exprs),
            span: exprs.span,
            ty: exprs.ty,
        }
    }

    fn reduce_irr(&mut self, env: Env, irr: Irreducible) -> Irreducible {
        let node = match irr.node {
            IrreducibleNode::Quote(exprs) => return self.reduce_exprs(env, exprs),
            IrreducibleNode::Lambda(param, body, _) => {
                let t = match self.types.get(&irr.ty) {
                    Type::Fun(t, _) => *t,
                    Type::Invalid => irr.ty,
                    _ => unreachable!(),
                };

                let new_name = self.names.fresh(irr.span, None);
                self.context.add(new_name, t);

                let closed = env.with(
                    param,
                    Irreducible {
                        node: IrreducibleNode::Quote(ExprSeq {
                            exprs: vec![Expr {
                                node: ExprNode::Produce(Value {
                                    node: ValueNode::Name(new_name),
                                    span: irr.span, // todo?
                                    ty: t,
                                }),
                                span: irr.span,
                                ty: t,
                            }],
                            span: irr.span,
                            ty: t,
                        }),
                        span: irr.span,
                        ty: t,
                    },
                );

                let body = self.reduce_irr(closed.clone(), *body);
                IrreducibleNode::Lambda(new_name, Box::new(body), closed)
            }
            irr => irr,
        };

        Irreducible {
            node,
            span: irr.span,
            ty: irr.ty,
        }
    }

    fn reduce_value(&mut self, env: &Env, value: Value) -> Irreducible {
        let node = match value.node {
            ValueNode::Invalid => IrreducibleNode::Invalid,
            ValueNode::Int(i) => IrreducibleNode::Integer(i),
            ValueNode::Name(name) => {
                if let Some(value) = env.lookup(&name) {
                    return value.clone();
                } else {
                    IrreducibleNode::Quote(ExprSeq {
                        span: value.span,
                        ty: value.ty,
                        exprs: vec![Expr {
                            node: ExprNode::Produce(Value {
                                node: ValueNode::Name(name),
                                span: value.span,
                                ty: value.ty,
                            }),
                            span: value.span,
                            ty: value.ty,
                        }],
                    })
                }
            }
        };

        Irreducible {
            node,
            span: value.span,
            ty: value.ty,
        }
    }
}
