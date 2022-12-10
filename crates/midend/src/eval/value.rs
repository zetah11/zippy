use common::mir::{Branch, BranchNode, Statement, StmtNode, Value, ValueNode};
use common::names::Name;
use common::Driver;

use super::Interpreter;

#[derive(Clone, Debug)]
pub struct ReducedValue {
    pub value: Value,

    /// The index of the frame this value originates. If the current frame index
    /// is equal or less, then this value is effectively "static" in that frame.
    /// But if the current frame index is higher, then this value originates
    /// from below us in the call stack, and so is dynamic.
    pub frame: usize,
}

impl ReducedValue {
    pub fn is_dynamic(&self, reference: usize) -> bool {
        !self.is_static(reference)
    }

    pub fn is_static(&self, reference: usize) -> bool {
        self.frame >= reference
    }
}

#[derive(Debug)]
pub enum Operation {
    Branch(Branch),
    Statement(Statement),
}

impl<D: Driver> Interpreter<'_, D> {
    pub(super) fn locally_static_value(&self, value: Value) -> ReducedValue {
        ReducedValue {
            value,
            frame: self.frame_index(),
        }
    }

    pub(super) fn frame_index(&self) -> usize {
        self.frames.len()
    }

    pub(super) fn get_args(&self, op: &Operation) -> Vec<Value> {
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

    pub(super) fn get_targets(&self, op: &Operation) -> Vec<Name> {
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
