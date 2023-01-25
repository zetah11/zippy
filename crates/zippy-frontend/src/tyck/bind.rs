use zippy_common::hir2::{Pat, PatNode, Type};
use zippy_common::message::Span;
use zippy_common::names2::Name;

use super::Typer;
use crate::resolved;

impl Typer<'_> {
    pub fn bind_pat(&mut self, pat: resolved::Pat, ty: Type) -> Pat {
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
                let id = self.assignable(pat.span, ty.clone(), uy.clone());
                let pat = self.bind_pat(*pat, uy);
                (PatNode::Coerce(Box::new(pat), id), ty)
            }

            PatNode::Wildcard => (PatNode::Wildcard, ty),

            PatNode::Invalid => (PatNode::Invalid, Type::Invalid),

            PatNode::Coerce(..) => unreachable!(),
        };

        Pat {
            node,
            span: pat.span,
            data: ty,
        }
    }

    pub fn bind_generic(
        &mut self,
        pat: resolved::Pat,
        params: &Vec<(Name, Span)>,
        ty: Type,
    ) -> Pat {
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
                let id = self.assignable(pat.span, ty.clone(), uy.clone());
                let pat = self.bind_generic(*pat, params, uy);
                (PatNode::Coerce(Box::new(pat), id), ty)
            }

            PatNode::Wildcard => (PatNode::Wildcard, ty),
            PatNode::Invalid => (PatNode::Invalid, ty),
            PatNode::Coerce(..) => unreachable!(),
        };

        Pat {
            node,
            span: pat.span,
            data: ty,
        }
    }

    pub fn bind_fresh(&mut self, pat: resolved::Pat) -> Pat {
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
            PatNode::Coerce(..) => unreachable!(),
        };

        Pat {
            node,
            span: pat.span,
            data: ty,
        }
    }

    pub fn save_type(&mut self, pat: &Pat, ty: &Type) {
        match &pat.node {
            PatNode::Name(name) => {
                self.unifier.defs.add(*name, ty.clone());
            }

            PatNode::Anno(pat, _) => self.save_type(pat, ty),

            PatNode::Tuple(a, b) => {
                self.messages.at(pat.span).tyck_tuple_type();
                self.save_type(a, &Type::Invalid);
                self.save_type(b, &Type::Invalid);
            }

            PatNode::Wildcard => {}
            PatNode::Invalid => {}
            PatNode::Coerce(..) => unreachable!(),
        }
    }
}
