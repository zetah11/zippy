use super::{Register, TypeId};
use crate::message::Span;
use crate::names::Name;

#[derive(Clone, Debug)]
pub struct Value {
    pub node: ValueNode,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ValueNode {
    Integer(i64),
    Register(Register),
    Name(Name),
}

#[derive(Clone, Debug)]
pub struct Target {
    pub node: TargetNode,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TargetNode {
    Register(Register),
    Name(Name),
}
