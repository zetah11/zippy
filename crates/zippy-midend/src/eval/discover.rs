use zippy_common::mir::{
    Block, BranchNode, StaticValueNode, StmtNode, Type, TypeId, Types, Value, ValueNode,
};
use zippy_common::names::Name;
use zippy_common::Driver;

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

            let ty = self.context.get(&name);
            visit_type(self.types, &mut worklist, &ty);

            if self.make_top_level(&name).is_some() {
                // annoyingly, even though `make_top_level` returns an immutable
                // reference, Rust treats it as if it has exclusive access to
                // `self` because the function takes `self` by mut ref.
                let block = self.blocks.get(&name).unwrap();
                visit_block(self.types, &mut worklist, block);
                self.worklist.insert(0, name);
            } else {
                // check if there is a top level with that name, and if so,
                // visit its type
                if let Some(value) = self.globals.get(&name) {
                    visit_type(self.types, &mut worklist, &value.ty);
                }
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
                StaticValueNode::Num(i) => {
                    self.globals.insert(
                        *name,
                        Value {
                            node: ValueNode::Num(i),
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

fn visit_block(types: &Types, worklist: &mut Vec<Name>, block: &Block) {
    fn name_of_value(value: &Value) -> Option<Name> {
        match &value.node {
            ValueNode::Name(name) => Some(*name),
            ValueNode::Num(_) | ValueNode::Invalid => None,
        }
    }

    visit_type(types, worklist, &block.ty);

    for stmt in block.stmts.iter() {
        visit_type(types, worklist, &stmt.ty);

        match &stmt.node {
            StmtNode::Apply { fun, args, .. } => {
                worklist.extend(args.iter().filter_map(name_of_value));
                worklist.push(*fun);
            }

            StmtNode::Coerce { of, from, .. } => {
                visit_type(types, worklist, from);
                worklist.push(*of);
            }

            StmtNode::Function { .. } => todo!(),
            StmtNode::Join { .. } => todo!(),
            StmtNode::Proj { .. } => todo!(),
            StmtNode::Tuple { .. } => todo!(),
        }
    }

    visit_type(types, worklist, &block.branch.ty);

    match &block.branch.node {
        BranchNode::Jump(..) => todo!(),
        BranchNode::Return(values) => worklist.extend(values.iter().filter_map(name_of_value)),
    }
}

fn visit_type(types: &Types, worklist: &mut Vec<Name>, ty: &TypeId) {
    match types.get(ty) {
        Type::Fun(ts, us) => {
            for ty in ts.iter().chain(us.iter()) {
                visit_type(types, worklist, ty);
            }
        }

        Type::Product(ts) => {
            for ty in ts.iter() {
                visit_type(types, worklist, ty);
            }
        }

        Type::Range(lo, hi) => {
            worklist.extend([*lo, *hi]);
        }

        Type::Invalid => {}
        Type::Number => {}
    }
}
