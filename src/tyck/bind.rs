use super::Typer;
use super::{Pat, PatNode, Type};

impl Typer {
    pub fn bind_pat(&mut self, pat: Pat, ty: Type) -> Pat<Type> {
        let (node, ty) = match pat.node {
            PatNode::Name(name) => {
                self.context.add(name, ty.clone());
                (PatNode::Name(name), ty)
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
}
