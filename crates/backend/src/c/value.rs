use common::mir::{StaticValue, StaticValueNode, Value, ValueNode};
use common::names::Name;

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

    pub fn emit_static_value(&mut self, ctx: Name, value: StaticValue) -> Option<String> {
        Some(match value.node {
            StaticValueNode::Int(i) => format!("{i}"),
            StaticValueNode::LateInit(block) => {
                let init = self.emit_block(ctx, Some(ctx), block);
                let init = init.join("\n\t");
                self.inits.push_str(&format!("\t{init}\n"));

                return None;
            }
        })
    }
}
