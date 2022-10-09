use super::{Actual, Name, Path, Resolver};
use crate::hir::{Pat, PatNode};

impl Resolver {
    pub fn resolve_pat(&mut self, pat: Pat) -> Pat<Name> {
        let node = match pat.node {
            PatNode::Name(name) => {
                let path = Path(self.context.clone(), Actual::Lit(name));
                // should never fail
                let name = self.names.lookup(&path).unwrap();
                PatNode::Name(name)
            }

            PatNode::Tuple(x, y) => {
                let x = Box::new(self.resolve_pat(*x));
                let y = Box::new(self.resolve_pat(*y));
                PatNode::Tuple(x, y)
            }

            PatNode::Wildcard => PatNode::Wildcard,
            PatNode::Invalid => PatNode::Invalid,
        };

        Pat {
            node,
            span: pat.span,
        }
    }
}
