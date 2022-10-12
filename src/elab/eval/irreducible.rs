use crate::message::Span;
use crate::mir::{ExprSeq, TypeId};
use crate::resolve::names::Name;

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

    Lambda(Name, Box<Irreducible>),
    Quote(ExprSeq),

    Invalid,
}
