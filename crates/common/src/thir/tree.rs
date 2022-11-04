pub use super::Type;

use crate::message::Span;
use crate::names::Name;

#[derive(Debug)]
pub struct Decls<Data = ()> {
    pub values: Vec<ValueDef<Data>>,
}

#[derive(Debug)]
pub struct ValueDef<Data = ()> {
    pub span: Span,
    pub pat: Pat<Data>,
    pub anno: Type,
    pub bind: Expr<Data>,
}

#[derive(Clone, Debug)]
pub struct Expr<Data = ()> {
    pub node: ExprNode<Data>,
    pub span: Span,
    pub data: Data,
}

#[derive(Clone, Debug)]
pub enum ExprNode<Data> {
    Name(Name),
    Int(i64),

    Lam(Pat<Data>, Box<Expr<Data>>),
    App(Box<Expr<Data>>, Box<Expr<Data>>),

    Anno(Box<Expr<Data>>, Span, Type),

    Tuple(Box<Expr<Data>>, Box<Expr<Data>>),

    Hole,
    Invalid,
}

#[derive(Clone, Debug)]
pub struct Pat<Data = ()> {
    pub node: PatNode<Data>,
    pub span: Span,
    pub data: Data,
}

#[derive(Clone, Debug)]
pub enum PatNode<Data> {
    Name(Name),
    Tuple(Box<Pat<Data>>, Box<Pat<Data>>),
    Wildcard,
    Invalid,
}