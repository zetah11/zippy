use log::trace;
use zippy_common::thir::{Because, TypeOrSchema};

use super::{Expr, ExprNode, Type, Typer};

impl Typer<'_> {
    /// Infer the type of an expression.
    pub fn infer(&mut self, ex: Expr) -> Expr<Type> {
        let (node, ty) = match ex.node {
            ExprNode::Name(name) => {
                let ty = self.context.get(&name).clone();
                let (ty, vars) = self.context.instantiate(&ty);

                if vars.is_empty() {
                    trace!("inferring monomorphic name");
                    (ExprNode::Name(name), ty)
                } else {
                    trace!("inferring polymorphic name");

                    let args = vars
                        .into_iter()
                        .map(|var| (ex.span, Type::mutable(var)))
                        .collect();

                    let expr = Box::new(Expr {
                        data: ty.clone(),
                        span: ex.span,
                        node: ExprNode::Name(name),
                    });

                    (ExprNode::Inst(expr, args), ty)
                }
            }

            ExprNode::Num(i) => {
                trace!("inferring int");
                let var = self.context.fresh();
                self.int_type(ex.span, Because::Unified(ex.span), Type::mutable(var));
                (ExprNode::Num(i), Type::mutable(var))
            }

            ExprNode::Tuple(a, b) => {
                trace!("inferring tuple");
                let t = self.context.fresh();
                let u = self.context.fresh();
                let a =
                    Box::new(self.check(Because::Inferred(ex.span, None), *a, Type::mutable(t)));
                let b =
                    Box::new(self.check(Because::Inferred(ex.span, None), *b, Type::mutable(u)));
                (
                    ExprNode::Tuple(a, b),
                    Type::Product(Box::new(Type::mutable(t)), Box::new(Type::mutable(u))),
                )
            }

            ExprNode::Lam(param, body) => {
                trace!("inferring lambda");
                let param = self.bind_fresh(param);
                let body = Box::new(self.infer(*body));

                let t = param.data.clone();
                let u = body.data.clone();

                let ty = Type::Fun(Box::new(t), Box::new(u));
                (ExprNode::Lam(param, body), ty)
            }

            ExprNode::App(fun, arg) => {
                trace!("inferring application");
                let fun = self.infer(*fun);
                let (t, u) = self.fun_type(ex.span, fun.data.clone());
                trace!("checking argument of app against {}", self.pretty(&t));
                let arg = self.check(Because::Inferred(fun.span, None), *arg, t);
                (ExprNode::App(Box::new(fun), Box::new(arg)), u)
            }

            ExprNode::Inst(fun, args) => {
                trace!("inferring instantiation");
                let ExprNode::Name(name) = fun.node else {
                    self.messages.at(fun.span).tyck_instantiate_non_name();
                    return self.infer(*fun);
                };

                let schema = match self.context.get(&name) {
                    schema @ TypeOrSchema::Schema(params, _) => {
                        if params.len() != args.len() {
                            self.messages.at(ex.span).tyck_instantiate_wrong_arity();
                        }

                        schema.clone()
                    }
                    TypeOrSchema::Type(ty) => {
                        let ty = ty.clone();
                        let pretty_type = self.pretty(&ty);
                        self.messages
                            .at(ex.span)
                            .tyck_instantiate_not_generic(Some(pretty_type));
                        return Expr {
                            node: ExprNode::Name(name),
                            span: ex.span,
                            data: ty,
                        };
                    }
                };

                let (ty, vars) = self.context.instantiate(&schema);

                for (var, (span, ty)) in vars.into_iter().zip(args.iter()) {
                    self.assignable(*span, Type::mutable(var), ty.clone());
                }

                let res = ExprNode::Inst(
                    Box::new(Expr {
                        node: ExprNode::Name(name),
                        span: fun.span,
                        data: ty.clone(),
                    }),
                    args,
                );

                (res, ty)
            }

            ExprNode::Anno(ex, anno_span, ty) => {
                trace!("inferring annotation");
                return self.check(Because::Annotation(anno_span), *ex, ty);
            }

            ExprNode::Hole => {
                trace!("inferring hole");
                (ExprNode::Hole, Type::mutable(self.context.fresh()))
            }

            ExprNode::Invalid => (ExprNode::Invalid, Type::Invalid),
        };

        Expr {
            node,
            span: ex.span,
            data: ty,
        }
    }
}
