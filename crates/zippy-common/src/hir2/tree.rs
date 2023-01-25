use super::{CoercionId, Type};
use crate::message::Span;
use crate::names2::Name;
use crate::Number;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decls {
    pub values: Vec<ValueDef>,
    pub types: Vec<TypeDef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueDef {
    pub span: Span,
    pub implicits: Vec<(Name, Span)>,
    pub pat: Pat,
    pub anno: Type,
    pub bind: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeDef {
    pub span: Span,
    pub pat: Pat,
    pub anno: Type,
    pub bind: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
    pub data: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExprNode {
    Name(Name),
    Num(Number),

    Lam(Pat, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Inst(Box<Expr>, Vec<(Span, Type)>),

    Anno(Box<Expr>, Span, Type),
    Coerce(Box<Expr>, CoercionId),

    Tuple(Box<Expr>, Box<Expr>),

    Hole,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pat {
    pub node: PatNode,
    pub span: Span,
    pub data: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatNode {
    Name(Name),
    Tuple(Box<Pat>, Box<Pat>),
    Anno(Box<Pat>, Type),
    Coerce(Box<Pat>, CoercionId),
    Wildcard,
    Invalid,
}
