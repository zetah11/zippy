use super::names::Name;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Operand {
    Register(Register),
    Immediate(Immediate),
    Memory(Address),
    Location(Name),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Address {
    pub reg: Option<Register>,
    pub offset: Option<Register>,
    pub scale: Scale,
    pub displacement: Option<i32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Scale {
    #[default]
    One,
    Two,
    Four,
    Eight,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Immediate {
    Imm8(u8),
    Imm16(u16),
    Imm32(u32),
    Imm64(u64),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rdi,
    Rsi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}
