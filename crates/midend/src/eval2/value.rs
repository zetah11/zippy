use common::mir::{Branch, BranchNode, Statement, StmtNode, Value, ValueNode};
use common::names::Name;

use super::Interpreter;

#[derive(Clone, Debug)]
pub enum ReducedValue {
    Static(Value),
    Dynamic(Value),
}

impl ReducedValue {
    pub fn is_dynamic(&self) -> bool {
        !self.is_static()
    }

    pub fn is_static(&self) -> bool {
        match self {
            Self::Static(_) => true,
            Self::Dynamic(_) => false,
        }
    }
}

#[derive(Debug)]
pub enum Operation {
    Branch(Branch),
    Statement(Statement),
}

impl Interpreter<'_> {
    pub fn get_args(&self, op: &Operation) -> Vec<Value> {
        match op {
            Operation::Branch(branch) => match &branch.node {
                BranchNode::Jump(_, arg) => vec![arg.clone()],
                BranchNode::Return(args) => args.clone(),
            },

            Operation::Statement(stmt) => match &stmt.node {
                StmtNode::Apply { args, fun, .. } => {
                    let mut res = args.clone();
                    let span = stmt.span;
                    let ty = self.context.get(fun);

                    res.insert(
                        0,
                        Value {
                            node: ValueNode::Name(*fun),
                            span,
                            ty,
                        },
                    );

                    res
                }

                StmtNode::Function { .. } => todo!(),
                StmtNode::Join { .. } => todo!(),

                StmtNode::Proj { of, .. } => {
                    let span = stmt.span;
                    let ty = self.context.get(of);
                    vec![Value {
                        node: ValueNode::Name(*of),
                        span,
                        ty,
                    }]
                }
                StmtNode::Tuple { values, .. } => values.clone(),
            },
        }
    }

    pub fn get_targets(&self, op: &Operation) -> Vec<Name> {
        match op {
            Operation::Branch(branch) => match &branch.node {
                BranchNode::Jump(..) => todo!(),
                BranchNode::Return(..) => Vec::new(),
            },

            Operation::Statement(stmt) => match &stmt.node {
                StmtNode::Apply { names, .. } => names.clone(),

                StmtNode::Function { .. } => todo!(),
                StmtNode::Join { .. } => todo!(),

                StmtNode::Proj { name, .. } => vec![*name],
                StmtNode::Tuple { name, .. } => vec![*name],
            },
        }
    }
}
