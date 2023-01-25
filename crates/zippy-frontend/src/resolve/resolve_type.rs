use zippy_common::names2::{Name, NamePart};

use super::Resolver;
use crate::resolved::{Expr, Pat, PatNode, Type, TypeNode, ValueDef};
use crate::unresolved;

impl Resolver<'_> {
    pub fn resolve_type(&mut self, values: &mut Vec<ValueDef>, ty: unresolved::Type) -> Type {
        let node = match ty.node {
            unresolved::TypeNode::Name(name) => match self.lookup(ty.span, name) {
                Some(name) => TypeNode::Name(name),
                None => TypeNode::Invalid,
            },

            unresolved::TypeNode::Product(t, u) => {
                let t = Box::new(self.resolve_type(values, *t));
                let u = Box::new(self.resolve_type(values, *u));

                TypeNode::Product(t, u)
            }

            unresolved::TypeNode::Fun(t, u) => {
                let t = Box::new(self.resolve_type(values, *t));
                let u = Box::new(self.resolve_type(values, *u));

                TypeNode::Fun(t, u)
            }

            unresolved::TypeNode::Range(lo, hi) => {
                let lo = self.resolve_expr(values, *lo);
                let hi = self.resolve_expr(values, *hi);

                let lo = self.lift_expr(values, lo);
                let hi = self.lift_expr(values, hi);

                TypeNode::Range(lo, hi)
            }

            unresolved::TypeNode::Type => TypeNode::Type,
            unresolved::TypeNode::Wildcard => TypeNode::Wildcard,
            unresolved::TypeNode::Invalid => TypeNode::Invalid,
        };

        Type {
            node,
            span: ty.span,
        }
    }

    /// Lift an expression from a range bound into a top-level binding, and
    /// return its name.
    fn lift_expr(&mut self, into: &mut Vec<ValueDef>, ex: Expr) -> Name {
        let span = ex.span;
        let name = Name::new(self.common_db(), self.context.1, NamePart::Spanned(span));

        into.push(ValueDef {
            pat: Pat {
                node: PatNode::Name(name),
                span,
            },
            implicits: vec![],
            anno: Type {
                node: TypeNode::Number,
                span,
            },
            bind: ex,
            span,
        });

        name
    }
}
