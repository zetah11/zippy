use super::Encoder;

#[derive(Clone, Debug)]
pub struct Relocation {
    pub kind: RelocationKind,
    pub at: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RelocationKind {
    /// 32-bit relocation, relative to current instruction
    Relative,

    /// 32-bit relocation, relative to next instruction
    RelativeNext,

    /// 64-bit absolute relocation.
    Absolute,
}

impl Encoder {
    pub fn perform_relocations(&mut self) {
        for (name, relocations) in self.relocations.drain() {
            let (address, _) = self.addresses.get(&name).copied().unwrap();
            for Relocation { kind, at } in relocations.iter() {
                match kind {
                    RelocationKind::Absolute => {
                        assert!(address <= u64::MAX as usize);
                        self.code
                            .splice(*at..*at + 8, (address as u64).to_le_bytes());
                    }

                    RelocationKind::Relative => {
                        let address = isize::try_from(address).unwrap();
                        let curr = isize::try_from(at.checked_sub(1).unwrap()).unwrap();
                        let diff = i32::try_from(address - curr).unwrap();
                        self.code.splice(*at..*at + 4, diff.to_le_bytes());
                    }

                    RelocationKind::RelativeNext => {
                        let address = isize::try_from(address).unwrap();
                        let next = isize::try_from(at.checked_add(4).unwrap()).unwrap();
                        let diff = i32::try_from(address - next).unwrap();
                        self.code.splice(*at..*at + 4, diff.to_le_bytes());
                    }
                }
            }
        }
    }
}
