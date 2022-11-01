use super::super::repr::Name;
use super::Encoder;

impl Encoder {
    pub fn encode_constant(&mut self, name: Name, data: Vec<u8>) {
        let offset = self.data.len();
        let size = data.len();

        self.data.extend(data);

        assert!(self.constants.insert(name, (offset, size)).is_none());
    }
}
