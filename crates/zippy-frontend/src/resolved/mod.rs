//! The resolved HIR is a tree-like representation of the code, where most names
//! have been resolved to some globally unique ID representing the actual name.
//! This essentially means that all names live in a global scope, which
//! simplifies many later passes.

use zippy_common::message::Span;
use zippy_common::names2::Name;
use zippy_common::Number;

#[salsa::tracked]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Decls {
    #[return_ref]
    pub values: Vec<ValueDef>,

    #[return_ref]
    pub types: Vec<TypeDef>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeDef {
    pub span: Span,
    pub pat: Pat,
    pub anno: Type,
    pub bind: Type,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ValueDef {
    pub span: Span,
    pub pat: Pat,
    pub implicits: Vec<Name>,
    pub anno: Type,
    pub bind: Expr,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExprNode {
    Name(Name),
    Num(Number),

    Lam(Pat, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Inst(Box<Expr>, Vec<Type>),

    Tuple(Box<Expr>, Box<Expr>),

    Anno(Box<Expr>, Type),

    Hole,
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pat {
    pub node: PatNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PatNode {
    Name(Name),
    Tuple(Box<Pat>, Box<Pat>),
    Anno(Box<Pat>, Type),
    Wildcard,
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Type {
    pub node: TypeNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeNode {
    Name(Name),
    Range(Box<Expr>, Box<Expr>),
    Fun(Box<Type>, Box<Type>),
    Product(Box<Type>, Box<Type>),
    Type,
    Wildcard,
    Invalid,
}
