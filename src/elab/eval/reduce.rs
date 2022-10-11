use crate::message::Span;
use crate::mir::{Expr, ExprNode, ExprSeq, TypeId, Value};
use crate::Driver;

use super::Env;
use super::Irreducible;
use super::Lowerer;

impl<D: Driver> Lowerer<'_, D> {
    pub fn reduce_exprs(&mut self, env: Env, exprs: ExprSeq) -> Irreducible {
        let mut env = env.child();

        let mut new_exprs = ExprSeq::default();

        for expr in exprs.exprs {
            let node = match expr.node {
                ExprNode::Produce(Value::Int(i)) => return Irreducible::Integer(i),
                ExprNode::Produce(Value::Invalid) => return Irreducible::Invalid,
                ExprNode::Produce(Value::Name(name)) => {
                    if let Some(value) = env.lookup(&name) {
                        return value.clone();
                    } else {
                        ExprNode::Produce(Value::Name(name))
                    }
                }

                ExprNode::Jump(..) | ExprNode::Join { .. } => todo!(),

                ExprNode::Function { name, param, body } => {
                    env.set(name, Irreducible::Lambda(param, body.clone(), env.clone()));
                    ExprNode::Function { name, param, body }
                }

                ExprNode::Apply { name, fun, arg } => {
                    if let Some(Irreducible::Lambda(param, body, closed)) = env.lookup(&fun) {
                        let arg = self.reduce_value(expr.span, expr.typ, &env, arg);

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
                        .map(|value| self.reduce_value(expr.span, expr.typ, &env, value))
                        .collect();
                    env.set(name, Irreducible::Tuple(values));
                    continue;
                }

                ExprNode::Proj { name, of, at } => {
                    if let Some(Irreducible::Tuple(values)) = env.lookup(&of) {
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
                typ: expr.typ,
            })
        }

        Irreducible::Quote(new_exprs)
    }

    fn reduce_value(&mut self, span: Span, typ: TypeId, env: &Env, value: Value) -> Irreducible {
        match value {
            Value::Int(i) => Irreducible::Integer(i),
            Value::Invalid => Irreducible::Invalid,
            Value::Name(name) => {
                if let Some(value) = env.lookup(&name) {
                    value.clone()
                } else {
                    Irreducible::Quote(ExprSeq {
                        exprs: vec![Expr {
                            node: ExprNode::Produce(Value::Name(name)),
                            span,
                            typ, // todo
                        }],
                    })
                }
            }
        }
    }
}
