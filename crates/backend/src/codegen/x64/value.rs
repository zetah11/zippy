use common::lir::{Value, ValueNode};
use common::names::Name;

use super::{Constraints, Lowerer};
use crate::asm::AllocConstraints;

impl Lowerer<'_> {
    pub fn lower_constant(&mut self, name: Name, value: Value) {
        let ty = self.program.context.get(&name);
        let size = Constraints::sizeof(&self.program.types, &ty);

        let ValueNode::Integer(i) = value.node else { unreachable!() };
        let bytes = i.to_le_bytes();

        self.set_label(name);

        // 0 -> 1 here, should it do that?
        let size = size.next_power_of_two();
        self.asm.db(&bytes[..size]).unwrap();
    }
}
