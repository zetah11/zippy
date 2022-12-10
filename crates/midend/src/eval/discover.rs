use common::mir::{Block, BranchNode, StaticValueNode, StmtNode, Value, ValueNode};
use common::names::Name;
use common::Driver;

use super::Interpreter;

impl<D: Driver> Interpreter<'_, D> {
    /// Find all the blocks and values reachable from the entry point, and
    /// construct a worklist of items to be partially evaluated.
    pub fn discover_from_entry(&mut self, entry: Name) {
        let mut worklist = vec![entry];

        while let Some(name) = worklist.pop() {
            if self.worklist.contains(&name) {
                panic!("cycle! what to do");
            }

            if let Some(block) = self.make_top_level(&name) {
                visit_block(&mut worklist, block);
                self.worklist.insert(0, name);
            }
        }
    }

    /// Collect all the blocks and values in the program, and construct a
    /// worklist of items to be partially evaluated.
    pub fn discover_all(&mut self) {
        todo!()
    }

    /// Initialize the top-level item with the given name, and return its block.
    /// Returns `None` if the name is not a top-level item, or if it has no
    /// associated block.
    fn make_top_level(&mut self, name: &Name) -> Option<&Block> {
        if let Some((params, block)) = self.decls.functions.remove(name) {
            self.functions.insert(*name, params);
            self.blocks.insert(*name, block);
            self.blocks.get(name)
        } else if let Some(value) = self.decls.values.remove(name) {
            match value.node {
                StaticValueNode::Int(i) => {
                    self.globals.insert(
                        *name,
                        Value {
                            node: ValueNode::Int(i),
                            span: value.span,
                            ty: value.ty,
                        },
                    );
                    None
                }

                StaticValueNode::LateInit(block) => {
                    self.blocks.insert(*name, block);
                    self.blocks.get(name)
                }
            }
        } else {
            None
        }
    }
}

fn visit_block(worklist: &mut Vec<Name>, block: &Block) {
    fn name_of_value(value: &Value) -> Option<Name> {
        match &value.node {
            ValueNode::Name(name) => Some(*name),
            ValueNode::Int(_) | ValueNode::Invalid => None,
        }
    }

    for stmt in block.stmts.iter() {
        match &stmt.node {
            StmtNode::Apply { fun, args, .. } => {
                worklist.extend(args.iter().filter_map(name_of_value));
                worklist.push(*fun);
            }

            StmtNode::Function { .. } => todo!(),
            StmtNode::Join { .. } => todo!(),
            StmtNode::Proj { .. } => todo!(),
            StmtNode::Tuple { .. } => todo!(),
        }
    }

    match &block.branch.node {
        BranchNode::Jump(..) => todo!(),
        BranchNode::Return(values) => worklist.extend(values.iter().filter_map(name_of_value)),
    }
}
