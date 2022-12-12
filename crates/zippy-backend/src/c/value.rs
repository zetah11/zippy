use zippy_common::mir::{StaticValue, StaticValueNode, Value, ValueNode};
use zippy_common::names::Name;

use super::Emitter;
use crate::mangle::mangle;

impl Emitter<'_> {
    pub fn emit_value(&mut self, value: Value) -> String {
        match value.node {
            ValueNode::Num(i) => format!("{i}"),
            ValueNode::Name(name) => mangle(self.names, &name),
            ValueNode::Invalid => {
                let invalid = self.invalid();
                let ty = self.typename(&value.ty);
                format!("({ty}) {invalid}()")
            }
        }
    }

    pub fn emit_static_value(&mut self, ctx: Name, value: StaticValue) -> Option<String> {
        Some(match value.node {
            StaticValueNode::Num(i) => format!("{i}"),
            StaticValueNode::LateInit(block) => {
                let init = self.emit_block(ctx, Some(ctx), block);
                let init = init.join("\n\t");
                self.inits.push_str(&format!("\t{init}\n"));

                return None;
            }
        })
    }
}
