use zippy_common::hir2::{self, Because, Type};

use super::Typer;
use crate::resolved;

impl Typer<'_> {
    pub fn infer(&mut self, expr: &resolved::Expr) -> hir2::Expr {
        let (node, ty) = match &expr.node {
            resolved::ExprNode::Name(name) => {
                let (ty, vars) = self.context.get_instantiated(name);

                if vars.is_empty() {
                    (hir2::ExprNode::Name(*name), ty)
                } else {
                    let args = vars
                        .into_iter()
                        .map(|var| (expr.span, Type::mutable(var)))
                        .collect();

                    (hir2::ExprNode::Inst(*name, args), ty)
                }
            }

            resolved::ExprNode::App(fun, arg) => {
                let fun = Box::new(self.infer(fun));
                let (t, u) =
                    self.type_function(Because::Called(arg.span), expr.span, fun.data.clone());
                let arg = Box::new(self.check(Because::Argument(fun.span), arg, t));
                (hir2::ExprNode::App(fun, arg), u)
            }

            resolved::ExprNode::Inst(fun, args) => {
                let resolved::ExprNode::Name(name) = &fun.node else {
                    self.messages.at(expr.span).tyck_instantiate_non_name();
                    return self.infer(fun);
                };

                let (ty, vars) = self.context.get_instantiated(name);

                if vars.is_empty() {
                    // TODO: pretty-print type
                    self.messages
                        .at(expr.span)
                        .tyck_instantiate_not_generic(None::<&str>);
                    (hir2::ExprNode::Name(*name), ty)
                } else {
                    if vars.len() != args.len() {
                        // TODO: add expected/actual number of args
                        self.messages.at(expr.span).tyck_instantiate_wrong_arity();
                    }

                    let mut new_args = Vec::with_capacity(args.len());
                    for (var, ty) in vars.into_iter().zip(args.iter()) {
                        let span = ty.span;
                        let ty = self.lower_type(ty, hir2::Mutability::Mutable);
                        self.equate(span, Type::mutable(var), ty.clone());
                        new_args.push((span, ty));
                    }

                    let expr = hir2::ExprNode::Inst(*name, new_args);

                    (expr, ty)
                }
            }

            resolved::ExprNode::Anno(expr, ty) => {
                let span = ty.span;
                let ty = self.lower_type(ty, hir2::Mutability::Mutable);
                return self.check(Because::Annotation(span), expr, ty);
            }

            resolved::ExprNode::Invalid => (hir2::ExprNode::Invalid, Type::Invalid),

            resolved::ExprNode::Hole
            | resolved::ExprNode::Num(_)
            | resolved::ExprNode::Lam(..)
            | resolved::ExprNode::Tuple(..) => {
                self.messages.at(expr.span).tyck_ambiguous();
                (hir2::ExprNode::Invalid, Type::Invalid)
            }
        };

        hir2::Expr {
            node,
            span: expr.span,
            data: ty,
        }
    }
}
