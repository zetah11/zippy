use super::Resolver;
use crate::resolved::{Pat, PatNode};
use crate::unresolved;

impl Resolver<'_> {
    pub fn resolve_pat(&mut self, pat: unresolved::Pat) -> Pat {
        let node = match pat.node {
            unresolved::PatNode::Name(name) => {
                PatNode::Name(self.lookup(pat.span, name).expect("undeclared pattern"))
            }

            unresolved::PatNode::Tuple(a, b) => {
                let a = Box::new(self.resolve_pat(*a));
                let b = Box::new(self.resolve_pat(*b));

                PatNode::Tuple(a, b)
            }

            unresolved::PatNode::Anno(pat, ty) => {
                let pat = Box::new(self.resolve_pat(*pat));
                let ty = self.resolve_type(ty);

                PatNode::Anno(pat, ty)
            }

            unresolved::PatNode::Wildcard => PatNode::Wildcard,
            unresolved::PatNode::Invalid => PatNode::Invalid,
        };

        Pat {
            node,
            span: pat.span,
        }
    }
}
