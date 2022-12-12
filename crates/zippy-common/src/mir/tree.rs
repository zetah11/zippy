use std::collections::HashMap;

use super::TypeId;
use crate::message::Span;
use crate::names::Name;
use crate::Number;

#[derive(Debug, Default)]
pub struct Decls {
    pub defs: Vec<ValueDef>,

    pub values: HashMap<Name, StaticValue>,
    pub functions: HashMap<Name, (Vec<Name>, Block)>,
}

impl Decls {
    pub fn new(defs: Vec<ValueDef>) -> Self {
        Self {
            defs,
            values: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ValueDef {
    pub span: Span,
    pub name: Name,
    pub bind: Block,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Statement>,
    pub branch: Branch,
    pub span: Span,
    pub ty: TypeId,
}

impl Block {
    pub fn new(span: Span, ty: TypeId, exprs: Vec<Statement>, branch: Branch) -> Self {
        Self {
            stmts: exprs,
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
    Return(Vec<Value>),
    Jump(Name, Value),
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub node: StmtNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug)]
pub enum StmtNode {
    Join {
        name: Name,
        param: Name,
        body: Block,
    },
    Function {
        name: Name,
        params: Vec<Name>,
        body: Block,
    },
    Apply {
        names: Vec<Name>,
        fun: Name,
        args: Vec<Value>,
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

/// A static value is one that is alive for the entire duration of the program.
#[derive(Clone, Debug)]
pub struct StaticValue {
    pub node: StaticValueNode,
    pub span: Span,
    pub ty: TypeId,
}

impl StaticValue {
    pub fn needs_late_init(&self) -> bool {
        match &self.node {
            StaticValueNode::Num(_) => false,
            StaticValueNode::LateInit(_) => true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum StaticValueNode {
    Num(Number),
    LateInit(Block),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Value {
    pub node: ValueNode,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValueNode {
    Num(Number),
    Name(Name),
    Invalid,
}
