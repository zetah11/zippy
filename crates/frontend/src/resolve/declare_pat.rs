use common::hir::{Pat, PatNode};

use super::Actual;
use super::Resolver;

impl Resolver {
    pub fn declare_pat(&mut self, pat: &Pat) {
        match &pat.node {
            PatNode::Name(name) => {
                self.declare_name(pat.span, Actual::Lit(name.clone()));
            }
            PatNode::Tuple(x, y) => {
                self.declare_pat(x);
                self.declare_pat(y);
            }
            PatNode::Anno(pat, _ty) => {
                self.declare_pat(pat);
            }
            PatNode::Wildcard | PatNode::Invalid => (),
        }
    }
}
