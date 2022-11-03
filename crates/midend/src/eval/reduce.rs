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

                ExprNode::Function { name, params, body } => {
                    let body_irr = self.reduce_exprs(env.clone(), body.clone());
                    env.set(
                        name,
                        Irreducible {
                            node: IrreducibleNode::Lambda(params.clone(), Box::new(body_irr)),
                            span: expr.span,
                            ty: expr.ty,
                        },
                    );

                    ExprNode::Function { name, params, body }
                }

                ExprNode::Apply { names, fun, args } => {
                    let reduced_args: Vec<_> = args
                        .iter()
                        .cloned()
                        .map(|arg| self.reduce_value(&env, arg))
                        .collect();

                    if let Some(Irreducible {
                        node: IrreducibleNode::Lambda(params, body),
                        ..
                    }) = self.lookup(&env, &fun)
                    {
                        let mut child_env = env.clone();
                        for (param, arg) in params.iter().zip(reduced_args) {
                            child_env = child_env.with(*param, arg);
                        }

                        let result = self.reduce_irr(child_env, *body.clone());
                        match result.node {
                            IrreducibleNode::Tuple(values) => {
                                assert!(names.len() == values.len());
                                for (name, value) in names.iter().zip(values) {
                                    env.set(*name, value);
                                }
                            }

                            IrreducibleNode::Invalid => {
                                for name in names.iter() {
                                    env.set(*name, result.clone());
                                }
                            }

                            _ => {
                                assert!(names.len() == 1);
                                env.set(names[0], result);
                            }
                        }
                    }

                    ExprNode::Apply { names, fun, args }
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
            BranchNode::Return(values) => 'branch: {
                let mut tup = Vec::with_capacity(values.len());
                for value in values.iter() {
                    match &value.node {
                        ValueNode::Int(i) => tup.push(Irreducible {
                            node: IrreducibleNode::Integer(*i),
                            span: value.span,
                            ty: value.ty,
                        }),

                        ValueNode::Invalid => tup.push(Irreducible {
                            node: IrreducibleNode::Invalid,
                            span: value.span,
                            ty: value.ty,
                        }),

                        ValueNode::Name(name) => {
                            if let Some(value) = self.lookup(&env, name) {
                                tup.push(value.clone());
                            } else {
                                break 'branch BranchNode::Return(values);
                            }
                        }
                    }
                }

                if tup.len() == 1 {
                    return tup.remove(0);
                } else {
                    return Irreducible {
                        node: IrreducibleNode::Tuple(tup),
                        span: exprs.branch.span,
                        ty: exprs.branch.ty,
                    };
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
            IrreducibleNode::Lambda(params, body) => {
                let t = match self.types.get(&irr.ty) {
                    Type::Fun(t, _) => t.clone(),
                    Type::Invalid => vec![irr.ty; params.len()],
                    _ => unreachable!(),
                };

                let new_names: Vec<_> = t
                    .iter()
                    .copied()
                    .map(|t| {
                        let name = self.names.fresh(irr.span, None);
                        self.context.add(name, t);
                        name
                    })
                    .collect();

                let mut closed = env;
                for (param, ty) in params.iter().zip(t) {
                    closed = closed.with(
                        *param,
                        Irreducible {
                            node: IrreducibleNode::Quote(ExprSeq {
                                exprs: vec![],
                                branch: Branch {
                                    node: BranchNode::Return(vec![Value {
                                        node: ValueNode::Name(*param),
                                        span: irr.span,
                                        ty,
                                    }]),
                                    span: irr.span,
                                    ty,
                                },
                                span: irr.span,
                                ty,
                            }),
                            span: irr.span,
                            ty,
                        },
                    );
                }

                let body = self.reduce_irr(closed, *body);
                IrreducibleNode::Lambda(new_names, Box::new(body))
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
                            node: BranchNode::Return(vec![Value {
                                node: ValueNode::Name(name),
                                span: value.span,
                                ty: value.ty,
                            }]),
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
