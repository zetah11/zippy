use std::ops::BitOr;

use super::super::repr::{Address, Register, Scale};
use super::Encoder;

#[derive(Clone, Copy, Debug)]
pub struct Rex {
    data: u8,
}

impl Rex {
    pub const NONE: Self = Self { data: 0 };
    pub const WIDE: Self = Self { data: 0b1000 };
    pub const MODRM_REG: Self = Self { data: 0b0100 };
    pub const SIB_INDEX: Self = Self { data: 0b0010 };
    pub const SIB_BASE: Self = Self { data: 0b0001 };

    pub fn from_base(set: bool) -> Self {
        if set {
            Self::SIB_BASE
        } else {
            Self::NONE
        }
    }

    pub fn from_index(set: bool) -> Self {
        if set {
            Self::SIB_INDEX
        } else {
            Self::NONE
        }
    }

    pub fn from_reg(set: bool) -> Self {
        if set {
            Self::MODRM_REG
        } else {
            Self::NONE
        }
    }
}

impl BitOr for Rex {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        if (self.data & rhs.data) != 0 {
            unreachable!()
        }

        Self {
            data: self.data | rhs.data,
        }
    }
}

impl TryFrom<Rex> for u8 {
    type Error = ();

    fn try_from(value: Rex) -> Result<Self, Self::Error> {
        if value.data == 0 {
            Err(())
        } else {
            assert!(value.data <= 0x0f);
            Ok(0x40 | value.data)
        }
    }
}

#[allow(clippy::unusual_byte_groupings)]
impl Encoder {
    /// Produce the X and B bits, the R/M bits, and any following bytes for the given address.
    pub fn encode_addr(&self, addr: Address) -> (Rex, u8, Vec<u8>) {
        match addr {
            // [rsp]
            Address {
                reg: Some(Register::Rsp),
                scale: _,
                offset: None,
                displacement: None,
            } => {
                let sib = 0b00_100_100;
                (Rex::NONE, 0x14, vec![sib])
            }

            // [rax]
            Address {
                reg: Some(reg),
                scale: Scale::One,
                offset: None,
                displacement: None,
            } => {
                let (rex, reg) = self.encode_reg(reg);
                (Rex::from_base(rex), 0x10 | reg, vec![])
            }

            // [2*rax]
            Address {
                reg: None,
                scale,
                offset: Some(offset),
                displacement,
            } => {
                let mut rest: Vec<_> = displacement
                    .unwrap_or(0)
                    .to_le_bytes()
                    .into_iter()
                    .collect();

                let (rex, offset) = self.encode_reg(offset);
                let sib = self.encode_scale(scale) | offset << 3 | 0b101;

                rest.insert(0, sib);

                (Rex::from_index(rex), 0x14, rest)
            }

            // [rax+2*rbx]
            Address {
                reg: Some(reg),
                scale,
                offset: Some(offset),
                displacement: None,
            } => {
                if reg == Register::Rbp {
                    self.encode_addr(Address {
                        reg: Some(reg),
                        scale,
                        offset: Some(offset),
                        displacement: Some(0),
                    })
                } else {
                    let (base, reg) = self.encode_reg(reg);
                    let (index, offset) = self.encode_reg(offset);
                    let sib = self.encode_scale(scale) | offset << 3 | reg;

                    (
                        Rex::from_base(base) | Rex::from_index(index),
                        0x14,
                        vec![sib],
                    )
                }
            }

            Address {
                reg: Some(reg),
                scale,
                offset: Some(offset),
                displacement: Some(displacement),
            } => {
                let mut rest: Vec<_> = displacement.to_le_bytes().into_iter().collect();

                let (base, reg) = self.encode_reg(reg);
                let (index, offset) = self.encode_reg(offset);

                let sib = self.encode_scale(scale) | offset << 3 | reg;
                rest.insert(0, sib);

                (
                    Rex::from_base(base) | Rex::from_index(index),
                    0b10_100_00,
                    rest,
                )
            }

            _ => unimplemented!(),
        }
    }

    /// Get the three-bit encoding of a register, and whether it is an extended register or not.
    pub fn encode_reg(&self, reg: Register) -> (bool, u8) {
        match reg {
            Register::Rax => (false, 0b000),
            Register::Rcx => (false, 0b001),
            Register::Rdx => (false, 0b010),
            Register::Rbx => (false, 0b011),
            Register::Rsp => (false, 0b100),
            Register::Rbp => (false, 0b101),
            Register::Rsi => (false, 0b110),
            Register::Rdi => (false, 0b111),

            Register::R8 => (true, 0b000),
            Register::R9 => (true, 0b001),
            Register::R10 => (true, 0b010),
            Register::R11 => (true, 0b011),
            Register::R12 => (true, 0b100),
            Register::R13 => (true, 0b101),
            Register::R14 => (true, 0b110),
            Register::R15 => (true, 0b111),
        }
    }

    fn encode_scale(&self, scale: Scale) -> u8 {
        (match scale {
            Scale::One => 0b00,
            Scale::Two => 0b01,
            Scale::Four => 0b10,
            Scale::Eight => 0b11,
        }) << 6
    }
}
