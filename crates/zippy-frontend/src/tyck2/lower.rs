use zippy_common::hir2::Type;

use super::Typer;
use crate::resolved;

/// Lower a [`resolved::Type`] into a HIR [`Type`], using the function `w` to
/// produce a type when a wildcard type is encountered.
pub fn lower_type<W>(w: &mut W, ty: &resolved::Type) -> Type
where
    W: FnMut() -> Type,
{
    match &ty.node {
        resolved::TypeNode::Name(name) => Type::Name(*name),
        resolved::TypeNode::Range(lo, hi) => Type::Range(*lo, *hi),

        resolved::TypeNode::Fun(t, u) => {
            let t = Box::new(lower_type(w, t));
            let u = Box::new(lower_type(w, u));
            Type::Fun(t, u)
        }

        resolved::TypeNode::Product(t, u) => {
            let t = Box::new(lower_type(w, t));
            let u = Box::new(lower_type(w, u));
            Type::Product(t, u)
        }

        resolved::TypeNode::Type => Type::Type,
        resolved::TypeNode::Number => Type::Number,
        resolved::TypeNode::Wildcard => w(),
        resolved::TypeNode::Invalid => Type::Invalid,
    }
}

impl Typer<'_> {
    pub fn lower_type(&mut self, ty: &resolved::Type) -> Type {
        lower_type(&mut || Type::mutable(self.context.fresh()), ty)
    }
}
