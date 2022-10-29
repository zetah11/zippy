use super::super::repr::{Name, Procedure};
use super::Encoder;

impl Encoder {
    pub fn encode_procedure(&mut self, name: Name, mut procedure: Procedure) {
        let at = self.code.len();

        for inst in procedure.prelude {
            self.encode_instruction(inst);
        }

        for name in procedure.block_order.iter() {
            let block = procedure.blocks.remove(name).unwrap();
            self.encode_block(*name, block);
        }

        let size = self.code.len() - at;
        assert!(self.addresses.insert(name, (at, size)).is_none());
    }
}
