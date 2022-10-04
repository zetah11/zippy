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

            PatNode::Invalid => PatNode::Invalid,
        };

        Pat {
            node,
            span: pat.span,
        }
    }
}
