use crate::mir::{Expr, ExprNode, ExprSeq, Value, ValueNode};
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
                    env.set(
                        name,
                        Irreducible {
                            node: IrreducibleNode::Lambda(param, body.clone(), env.clone()),
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
                        let result = self.reduce_exprs(closed.with(*param, arg), body.clone());
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

    fn reduce_value(&mut self, env: &Env, value: Value) -> Irreducible {
        let node = match value.node {
            ValueNode::Int(i) => IrreducibleNode::Integer(i),
            ValueNode::Invalid => IrreducibleNode::Invalid,
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
