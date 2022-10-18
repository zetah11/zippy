use std::collections::HashMap;

pub use allocation::Constraints;

mod allocation;
mod info;
mod interfere;
mod live;
mod priority;

use crate::lir::{Block, BlockId, Branch, Inst, Proc, Target, Value};
use allocation::{allocate, Allocation};

pub fn regalloc(constraints: &Constraints, proc: Proc) -> Proc {
    let allocation = allocate(&proc, constraints);
    let applier = Applier::new(allocation);
    applier.apply(proc)
}

struct Applier {
    allocation: Allocation,
}

impl Applier {
    pub fn new(allocation: Allocation) -> Self {
        Self { allocation }
    }

    pub fn apply(&self, mut proc: Proc) -> Proc {
        let mut worklist = vec![proc.entry];
        let mut blocks = HashMap::with_capacity(proc.blocks.len());

        while let Some(id) = worklist.pop() {
            let block = match proc.blocks.remove(&id) {
                Some(id) => id,
                None => continue,
            };

            let insts = block
                .insts
                .into_iter()
                .map(|inst| self.apply_inst(inst))
                .collect();

            let (branch, succs) = self.apply_branch(block.branch);

            worklist.extend(succs);
            blocks.insert(id, Block { insts, branch });
        }

        blocks.entry(proc.entry).and_modify(|block| {
            if self.allocation.frame_space > 0 {
                block
                    .insts
                    .insert(0, Inst::Reserve(self.allocation.frame_space));
                block.insts.push(Inst::Release(self.allocation.frame_space));
            }
        });

        Proc {
            blocks,
            entry: proc.entry,
            exit: proc.exit,
        }
    }

    fn apply_branch(&self, branch: Branch) -> (Branch, Vec<BlockId>) {
        match branch {
            Branch::Return(value) => (Branch::Return(self.apply_value(value)), Vec::new()),
            Branch::Jump(succ) => (Branch::Jump(succ), vec![succ]),
            Branch::JumpIf {
                conditional,
                left,
                right,
                consequence,
                alternative,
            } => {
                let left = self.apply_value(left);
                let right = self.apply_value(right);
                let res = Branch::JumpIf {
                    conditional,
                    left,
                    right,
                    consequence,
                    alternative,
                };
                (res, vec![consequence, alternative])
            }
        }
    }

    fn apply_inst(&self, inst: Inst) -> Inst {
        match inst {
            Inst::Crash => Inst::Crash,
            Inst::Reserve(res) => Inst::Reserve(res),
            Inst::Release(res) => Inst::Release(res),
            Inst::Move(target, value) => {
                let target = self.apply_target(target);
                let value = self.apply_value(value);
                Inst::Move(target, value)
            }
            Inst::Push(value) => Inst::Push(self.apply_value(value)),
            Inst::Pop(target) => Inst::Pop(self.apply_target(target)),
            Inst::Call(value) => Inst::Call(self.apply_value(value)),
        }
    }

    fn apply_value(&self, value: Value) -> Value {
        match value {
            Value::Integer(i) => Value::Integer(i),
            Value::Location(l) => Value::Location(l),
            Value::Register(r) => Value::Register(self.allocation.map.get(&r).copied().unwrap()),
        }
    }

    fn apply_target(&self, target: Target) -> Target {
        match target {
            Target::Location(l) => Target::Location(l),
            Target::Register(r) => Target::Register(self.allocation.map.get(&r).copied().unwrap()),
        }
    }
}
