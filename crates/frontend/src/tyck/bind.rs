use super::Typer;
use super::{Pat, PatNode, Type};

impl Typer {
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
