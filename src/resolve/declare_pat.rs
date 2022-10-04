use super::names::Actual;
use super::Resolver;
use crate::hir::{Pat, PatNode};

impl Resolver {
    pub fn declare_pat(&mut self, pat: &Pat) {
        match &pat.node {
            PatNode::Name(name) => {
                self.declare_name(pat.span, Actual::Lit(name.clone()));
            }
            PatNode::Invalid => (),
        }
    }
}
