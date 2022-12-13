use std::fmt::{self, Display};

use crate::message::Span;
use crate::Number;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BindId(pub(crate) usize);

impl Display for BindId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "s{}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct BindIdGenerator {
    bind_id: usize,
}

impl BindIdGenerator {
    pub fn new() -> Self {
        Self { bind_id: 0 }
    }

    pub fn fresh(&mut self) -> BindId {
        let id = BindId(self.bind_id);
        self.bind_id += 1;
        id
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Decls<Name = String> {
    pub values: Vec<ValueDef<Name>>,
    pub types: Vec<TypeDef<Name>>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeDef<Name = String> {
    pub span: Span,
    pub id: BindId,
    pub pat: Pat<Name>,
    pub anno: Type<Name>,
    pub bind: Type<Name>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ValueDef<Name = String> {
    pub span: Span,
    pub id: BindId,
    pub pat: Pat<Name>,
    pub implicits: Vec<(Name, Span)>,
    pub anno: Type<Name>,
    pub bind: Expr<Name>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Expr<Name = String> {
    pub node: ExprNode<Name>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExprNode<Name> {
    Name(Name),
    Num(Number),

    Lam(BindId, Pat<Name>, Box<Expr<Name>>),
    App(Box<Expr<Name>>, Box<Expr<Name>>),
    Inst(Box<Expr<Name>>, Vec<Type<Name>>),

    Tuple(Box<Expr<Name>>, Box<Expr<Name>>),

    Anno(Box<Expr<Name>>, Type<Name>),

    Hole,
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pat<Name = String> {
    pub node: PatNode<Name>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PatNode<Name> {
    Name(Name),
    Tuple(Box<Pat<Name>>, Box<Pat<Name>>),
    Anno(Box<Pat<Name>>, Type<Name>),
    Wildcard,
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Type<Name = String> {
    pub node: TypeNode<Name>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeNode<Name> {
    Name(Name),
    Range(Box<Expr<Name>>, Box<Expr<Name>>),
    Fun(Box<Type<Name>>, Box<Type<Name>>),
    Prod(Box<Type<Name>>, Box<Type<Name>>),
    Type,
    Wildcard,
    Invalid,
}
