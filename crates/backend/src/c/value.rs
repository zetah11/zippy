use common::mir::{Value, ValueNode};

use super::Emitter;
use crate::mangle::mangle;

impl Emitter<'_> {
    pub fn emit_value(&self, value: Value) -> String {
        match value.node {
            ValueNode::Int(i) => format!("{i}"),
            ValueNode::Name(name) => mangle(self.names, &name),
            ValueNode::Invalid => "0".to_owned(),
        }
    }
}
