use zippy_common::hir::{Pat, PatNode};

use super::{Actual, Name, Path, Resolver};

impl Resolver {
    pub fn resolve_pat(&mut self, pat: Pat) -> Pat<Name> {
        let node = match pat.node {
            PatNode::Name(name) => {
                let path = Path::new(self.context(), Actual::Lit(name));
                // should never fail
                let name = self.names.lookup(&path).unwrap();
                PatNode::Name(name)
            }

            PatNode::Tuple(x, y) => {
                let x = Box::new(self.resolve_pat(*x));
                let y = Box::new(self.resolve_pat(*y));
                PatNode::Tuple(x, y)
            }

            PatNode::Anno(pat, ty) => {
                let pat = Box::new(self.resolve_pat(*pat));
                let ty = self.resolve_type(ty);
                PatNode::Anno(pat, ty)
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
