use zippy_common::{
    hir2::{self, Type},
    names2::Name,
};

use super::Typer;
use crate::resolved;

impl Typer<'_> {
    pub fn bind_pat(&mut self, pat: &resolved::Pat, ty: Type) -> hir2::Pat {
        let node = match &pat.node {
            resolved::PatNode::Name(name) => {
                self.context.add(*name, ty.clone());
                hir2::PatNode::Name(*name)
            }

            resolved::PatNode::Tuple(a, b) => {
                let (t, u) = self.type_tuple(pat.span, ty.clone());
                let a = Box::new(self.bind_pat(a, t));
                let b = Box::new(self.bind_pat(b, u));
                hir2::PatNode::Tuple(a, b)
            }

            resolved::PatNode::Anno(pat, anno) => {
                let span = anno.span;
                let anno = self.lower_type(anno, hir2::Mutability::Mutable);
                self.equate(span, ty, anno.clone());
                return self.bind_pat(pat, anno);
            }

            resolved::PatNode::Wildcard => hir2::PatNode::Wildcard,
            resolved::PatNode::Invalid => hir2::PatNode::Invalid,
        };

        hir2::Pat {
            node,
            span: pat.span,
            data: ty,
        }
    }

    /// Bind a pattern parameterized by some number of type names to a type
    /// annotation. If the type parameter list is empty, this does the same as
    /// [`Self::bind_pat`].
    ///
    /// This also returns a list of every name mentioned by this pattern.
    pub fn bind_pat_schema(
        &mut self,
        pat: &resolved::Pat,
        ty: Type,
        implicits: &[Name],
    ) -> hir2::Pat {
        let node = match &pat.node {
            resolved::PatNode::Name(name) => {
                self.context
                    .add_schema(*name, implicits.to_vec(), ty.clone());
                hir2::PatNode::Name(*name)
            }

            resolved::PatNode::Tuple(a, b) => {
                let (t, u) = self.type_tuple(pat.span, ty.clone());
                let a = Box::new(self.bind_pat_schema(a, t, implicits));
                let b = Box::new(self.bind_pat_schema(b, u, implicits));
                hir2::PatNode::Tuple(a, b)
            }

            resolved::PatNode::Anno(pat, anno) => {
                let span = anno.span;
                let anno = self.lower_type(anno, hir2::Mutability::Mutable);
                self.equate(span, ty, anno.clone());
                return self.bind_pat_schema(pat, anno, implicits);
            }

            resolved::PatNode::Wildcard => hir2::PatNode::Wildcard,
            resolved::PatNode::Invalid => hir2::PatNode::Invalid,
        };

        hir2::Pat {
            node,
            span: pat.span,
            data: ty,
        }
    }
}
