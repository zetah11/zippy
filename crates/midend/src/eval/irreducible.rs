use common::message::Span;
use common::mir::{Block, TypeId};
use common::names::Name;

#[derive(Clone, Debug)]
pub(super) struct Irreducible {
    pub node: IrreducibleNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug)]
pub(super) enum IrreducibleNode {
    Integer(i64),
    Tuple(Vec<Irreducible>),

    Lambda(Vec<Name>, Box<Irreducible>),
    Quote(Block),

    Invalid,
}
