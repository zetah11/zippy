use super::super::repr::{Name, Procedure};
use super::Encoder;

impl Encoder {
    pub fn encode_procedure(&mut self, name: Name, mut procedure: Procedure) {
        let at = self.code.len();
        assert!(self.addresses.insert(name, at).is_none());

        for name in procedure.block_order.iter() {
            let block = procedure.blocks.remove(name).unwrap();
            self.encode_block(*name, block);
        }
    }
}
