use crate::mir::{Branch, BranchNode, Expr, ExprNode, ExprSeq, Type, Value, ValueNode};
use crate::Driver;

use super::{Env, Irreducible, IrreducibleNode, Lowerer};

impl<D: Driver> Lowerer<'_, D> {
    pub fn reduce_exprs(&mut self, env: Env, exprs: ExprSeq) -> Irreducible {
        let mut env = env.child();

        let mut new_exprs = Vec::new();

        for expr in exprs.exprs {
            let node = match expr.node {
                ExprNode::Join { .. } => todo!(),

                ExprNode::Function { name, param, body } => {
                    let body_irr = self.reduce_exprs(env.clone(), body.clone());
                    env.set(
                        name,
                        Irreducible {
                            node: IrreducibleNode::Lambda(param, Box::new(body_irr)),
                            span: expr.span,
                            ty: expr.ty,
                        },
                    );

                    ExprNode::Function { name, param, body }
                }

                ExprNode::Apply { name, fun, arg } => {
                    if let Some(Irreducible {
                        node: IrreducibleNode::Lambda(param, body),
                        ..
                    }) = self.lookup(&env, &fun)
                    {
                        let arg = self.reduce_value(&env, arg.clone());
                        let result = self.reduce_irr(env.with(*param, arg), *body.clone());

                        env.set(name, result);
                    }

                    ExprNode::Apply { name, fun, arg }
                }

                ExprNode::Tuple { name, values } => {
                    let new_values = values
                        .clone()
                        .into_iter()
                        .map(|value| self.reduce_value(&env, value))
                        .collect();
                    env.set(
                        name,
                        Irreducible {
                            node: IrreducibleNode::Tuple(new_values),
                            span: expr.span,
                            ty: expr.ty,
                        },
                    );

                    ExprNode::Tuple { name, values }
                }

                ExprNode::Proj { name, of, at } => {
                    if let Some(Irreducible {
                        node: IrreducibleNode::Tuple(values),
                        ..
                    }) = self.lookup(&env, &of)
                    {
                        env.set(name, values[at].clone());
                    }

                    ExprNode::Proj { name, of, at }
                }
            };

            new_exprs.push(Expr {
                node,
                span: expr.span,
                ty: expr.ty,
            })
        }

        let branch = match exprs.branch.node {
            BranchNode::Return(Value {
                node: ValueNode::Int(i),
                ..
            }) => {
                return Irreducible {
                    node: IrreducibleNode::Integer(i),
                    span: exprs.branch.span,
                    ty: exprs.branch.ty,
                };
            }
            BranchNode::Return(Value {
                node: ValueNode::Invalid,
                ..
            }) => {
                return Irreducible {
                    node: IrreducibleNode::Invalid,
                    span: exprs.branch.span,
                    ty: exprs.branch.ty,
                };
            }
            BranchNode::Return(Value {
                node: ValueNode::Name(name),
                span,
                ty,
            }) => {
                if let Some(value) = self.lookup(&env, &name) {
                    return value.clone();
                } else {
                    BranchNode::Return(Value {
                        node: ValueNode::Name(name),
                        span,
                        ty,
                    })
                }
            }
            BranchNode::Jump(to, arg) => BranchNode::Jump(to, arg),
        };

        Irreducible {
            node: IrreducibleNode::Quote(ExprSeq {
                exprs: new_exprs,
                branch: Branch {
                    node: branch,
                    span: exprs.branch.span,
                    ty: exprs.branch.ty,
                },
                span: exprs.span,
                ty: exprs.ty,
            }),
            span: exprs.span,
            ty: exprs.ty,
        }
    }

    pub fn reduce_irr(&mut self, env: Env, irr: Irreducible) -> Irreducible {
        let node = match irr.node {
            IrreducibleNode::Quote(exprs) => return self.reduce_exprs(env, exprs),
            IrreducibleNode::Lambda(param, body) => {
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
                            exprs: vec![],
                            branch: Branch {
                                node: BranchNode::Return(Value {
                                    node: ValueNode::Name(new_name),
                                    span: irr.span, // todo?
                                    ty: t,
                                }),
                                span: irr.span,
                                ty: t,
                            },
                            span: irr.span,
                            ty: t,
                        }),
                        span: irr.span,
                        ty: t,
                    },
                );

                let body = self.reduce_irr(closed, *body);
                IrreducibleNode::Lambda(new_name, Box::new(body))
            }

            IrreducibleNode::Tuple(irrs) => {
                let irrs = irrs
                    .into_iter()
                    .map(|irr| self.reduce_irr(env.clone(), irr))
                    .collect();
                IrreducibleNode::Tuple(irrs)
            }

            IrreducibleNode::Integer(i) => IrreducibleNode::Integer(i),
            IrreducibleNode::Invalid => IrreducibleNode::Invalid,
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
                if let Some(value) = self.lookup(env, &name) {
                    return value.clone();
                } else {
                    IrreducibleNode::Quote(ExprSeq {
                        exprs: vec![],
                        span: value.span,
                        ty: value.ty,
                        branch: Branch {
                            node: BranchNode::Return(Value {
                                node: ValueNode::Name(name),
                                span: value.span,
                                ty: value.ty,
                            }),
                            span: value.span,
                            ty: value.ty,
                        },
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
