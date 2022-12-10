use zippy_common::message::Span;
use zippy_common::names::Name;

use super::Typer;
use super::{Pat, PatNode, Type};

impl Typer<'_> {
    pub fn bind_pat(&mut self, pat: Pat, ty: Type) -> Pat<Type> {
        let (node, ty) = match pat.node {
            PatNode::Name(name) => {
                self.context.add(name, ty.clone());
                (PatNode::Name(name), ty)
            }

            PatNode::Tuple(x, y) => {
                let (t, u) = self.tuple_type(pat.span, ty);
                let x = Box::new(self.bind_pat(*x, t));
                let y = Box::new(self.bind_pat(*y, u));

                let ty = Type::Product(Box::new(x.data.clone()), Box::new(y.data.clone()));

                (PatNode::Tuple(x, y), ty)
            }

            PatNode::Anno(pat, uy) => {
                self.assignable(pat.span, ty, uy.clone());
                return self.bind_pat(*pat, uy);
            }

            PatNode::Wildcard => (PatNode::Wildcard, ty),

            PatNode::Invalid => (PatNode::Invalid, Type::Invalid),
        };

        Pat {
            node,
            span: pat.span,
            data: ty,
        }
    }

    pub fn bind_generic(&mut self, pat: Pat, params: &Vec<(Name, Span)>, ty: Type) -> Pat<Type> {
        if params.is_empty() {
            let pat = self.bind_pat(pat, ty);
            return pat;
        }

        let (node, ty) = match pat.node {
            PatNode::Name(name) => {
                self.context.add_schema(
                    name,
                    params.iter().map(|(name, _)| *name).collect(),
                    ty.clone(),
                );
                (PatNode::Name(name), ty)
            }

            PatNode::Tuple(x, y) => {
                let (t, u) = self.tuple_type(pat.span, ty);
                let x = Box::new(self.bind_generic(*x, params, t));
                let y = Box::new(self.bind_generic(*y, params, u));

                let ty = Type::Product(Box::new(x.data.clone()), Box::new(y.data.clone()));

                (PatNode::Tuple(x, y), ty)
            }

            PatNode::Anno(pat, uy) => {
                self.assignable(pat.span, ty, uy.clone());
                return self.bind_generic(*pat, params, uy);
            }

            PatNode::Wildcard => (PatNode::Wildcard, ty),
            PatNode::Invalid => (PatNode::Invalid, ty),
        };

        Pat {
            node,
            span: pat.span,
            data: ty,
        }
    }

    pub fn bind_fresh(&mut self, pat: Pat) -> Pat<Type> {
        let (node, ty) = match pat.node {
            PatNode::Name(name) => {
                let ty = self.context.fresh();
                self.context.add(name, Type::mutable(ty));
                (PatNode::Name(name), Type::mutable(ty))
            }

            PatNode::Tuple(x, y) => {
                let x = Box::new(self.bind_fresh(*x));
                let y = Box::new(self.bind_fresh(*y));

                let t = x.data.clone();
                let u = y.data.clone();

                (
                    PatNode::Tuple(x, y),
                    Type::Product(Box::new(t), Box::new(u)),
                )
            }

            PatNode::Anno(pat, ty) => return self.bind_pat(*pat, ty),

            PatNode::Wildcard => (PatNode::Wildcard, Type::mutable(self.context.fresh())),

            PatNode::Invalid => (PatNode::Invalid, Type::Invalid),
        };

        Pat {
            node,
            span: pat.span,
            data: ty,
        }
    }
}