use super::super::repr::{Block, Name};
use super::Encoder;

impl Encoder {
    pub fn encode_block(&mut self, name: Name, block: Block) {
        let at = self.code.len();
        assert!(self.addresses.insert(name, at).is_none());

        for inst in block.insts {
            self.encode_instruction(inst);
        }
    }
}
