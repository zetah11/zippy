use common::lir::TypeId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Location {
    Local(usize, TypeId),
    Global,
}
