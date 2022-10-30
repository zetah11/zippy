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
