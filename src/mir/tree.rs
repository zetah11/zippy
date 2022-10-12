use super::TypeId;
use crate::message::Span;
use crate::resolve::names::Name;

#[derive(Debug)]
pub struct Decls {
    pub values: Vec<ValueDef>,
}

#[derive(Debug)]
pub struct ValueDef {
    pub span: Span,
    pub name: Name,
    pub bind: ExprSeq,
}

#[derive(Clone, Debug)]
pub struct ExprSeq {
    pub exprs: Vec<Expr>,
    pub branch: Branch,
    pub span: Span,
    pub ty: TypeId,
}

impl ExprSeq {
    pub fn new(span: Span, ty: TypeId, exprs: Vec<Expr>, branch: Branch) -> Self {
        Self {
            exprs,
            branch,
            span,
            ty,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub node: BranchNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug)]
pub enum BranchNode {
    Return(Value),
    Jump(Name, Value),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug)]
pub enum ExprNode {
    Join {
        name: Name,
        param: Name,
        body: ExprSeq,
    },
    Function {
        name: Name,
        param: Name,
        body: ExprSeq,
    },
    Apply {
        name: Name,
        fun: Name,
        arg: Value,
    },
    Tuple {
        name: Name,
        values: Vec<Value>,
    },
    Proj {
        name: Name,
        of: Name,
        at: usize,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Value {
    pub node: ValueNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValueNode {
    Int(i64),
    Name(Name),
    Invalid,
}
