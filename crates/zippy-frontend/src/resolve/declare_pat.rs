use super::path::NamePart;
use super::Resolver;
use crate::unresolved::{Pat, PatNode};

impl Resolver<'_> {
    pub fn declare_pat(&mut self, pat: &Pat) {
        match &pat.node {
            PatNode::Name(name) => {
                self.declare(pat.span, NamePart::Source(*name));
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
