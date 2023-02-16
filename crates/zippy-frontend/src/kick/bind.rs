use super::kinds::Kind;
use super::Kinder;
use crate::resolved::{Pat, PatNode};

impl Kinder<'_> {
    /// Bind a pattern to a fresh kind, and return that kind.
    pub fn bind(&mut self, pat: &Pat) -> Kind {
        match &pat.node {
            PatNode::Name(name) => {
                let var = self.fresh();

                assert!(self.context.insert(*name, Kind::Var(var)).is_none());
                Kind::Var(var)
            }

            PatNode::Tuple(a, b) => {
                let a = self.bind(a);
                let b = self.bind(b);

                Kind::Product(Box::new(a), Box::new(b))
            }

            PatNode::Anno(a, ty) => {
                let kind = self.kind_from_type(ty.clone());
                let a = self.bind(a);

                self.unify(pat.span, kind.clone(), a);
                kind
            }

            PatNode::Wildcard | PatNode::Invalid => Kind::Var(self.fresh()),
        }
    }
}
