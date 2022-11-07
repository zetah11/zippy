#[derive(Debug)]
pub enum RelocationKind {
    /// 32-bit address relative to next instruction.
    RelativeNext,
}
