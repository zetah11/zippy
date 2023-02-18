use zippy_common::hir2::Type;

use super::Definer;
use crate::resolved::{Pat, PatNode};

impl Definer<'_> {
    pub fn bind_type(&mut self, pat: &Pat, ty: Type) {
        match (&pat.node, ty) {
            (PatNode::Name(name), ty) => {
                assert!(self.types.insert(*name, ty).is_none());
            }

            (PatNode::Anno(pat, _), ty) => {
                self.bind_type(pat, ty);
            }

            (PatNode::Tuple(..), _) => {
                // TODO: emit error message
            }

            (PatNode::Wildcard | PatNode::Invalid, _) => {}
        }
    }
}
