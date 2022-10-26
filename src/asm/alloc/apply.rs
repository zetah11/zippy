use std::collections::HashMap;

use super::allocation::Allocation;
use crate::lir::{BlockId, Branch, Instruction, ProcBuilder, Procedure, Register, Target, Value};

pub fn apply(allocation: Allocation, procedure: Procedure) -> Procedure {
    let mut applier = Applier::new(allocation);
    applier.apply(procedure)
}

struct Applier {
    allocation: Allocation,
    block_map: HashMap<BlockId, BlockId>,
}

impl Applier {
    pub fn new(allocation: Allocation) -> Self {
        Self {
            allocation,
            block_map: HashMap::new(),
        }
    }

    pub fn apply(&mut self, mut proc: Procedure) -> Procedure {
        println!("{:?}", self.allocation);

        for cont in proc.continuations.iter() {
            self.block_map.insert(*cont, *cont);
        }

        let mut builder = ProcBuilder::new(
            proc.params
                .iter()
                .copied()
                .map(|param| self.apply_reg(param))
                .collect(),
            proc.continuations.drain(..),
        );

        for id in proc.blocks.keys() {
            let new_id = builder.fresh_id();
            self.block_map.insert(*id, new_id);
        }

        let mut worklist = vec![proc.entry];

        let entry = self.map_block(proc.entry);
        let mut exits = Vec::with_capacity(proc.exits.len());

        while let Some(id) = worklist.pop() {
            let block = if proc.has_block(&id) {
                proc.get(&id)
            } else {
                continue;
            };

            let mut insts = Vec::with_capacity(block.insts.len());

            for inst in block.insts.clone() {
                insts.push(self.apply_inst(proc.get_instruction(inst).clone()));
            }

            let (branch, next) = self.apply_branch(proc.get_branch(block.branch).clone());
            worklist.extend(next);

            let new_id = self.map_block(id);
            builder.add(
                new_id,
                block.param.map(|reg| self.apply_reg(reg)),
                insts,
                branch,
            );

            if proc.exits.contains(&id) {
                exits.push(new_id);
            }
        }

        builder.build(entry, exits)
    }

    fn apply_branch(&self, branch: Branch) -> (Branch, Vec<BlockId>) {
        match branch {
            Branch::Return(cont, value) => (
                Branch::Return(self.map_block(cont), self.apply_value(value)),
                vec![],
            ),
            Branch::Jump(succ, arg) => (
                Branch::Jump(
                    self.map_block(succ),
                    arg.map(|value| self.apply_value(value)),
                ),
                vec![succ],
            ),
            Branch::JumpIf {
                left,
                cond,
                right,
                then: (then, then_arg),
                elze: (elze, elze_arg),
            } => {
                let left = self.apply_value(left);
                let right = self.apply_value(right);
                let then_arg = then_arg.map(|value| self.apply_value(value));
                let elze_arg = elze_arg.map(|value| self.apply_value(value));
                let res = Branch::JumpIf {
                    left,
                    cond,
                    right,
                    then: (self.map_block(then), then_arg),
                    elze: (self.map_block(elze), elze_arg),
                };

                (res, vec![then, elze])
            }
            Branch::Call(fun, args, conts) => {
                let fun = self.apply_value(fun);
                let args = args.into_iter().map(|arg| self.apply_value(arg)).collect();
                (
                    Branch::Call(
                        fun,
                        args,
                        conts
                            .iter()
                            .copied()
                            .map(|block| self.map_block(block))
                            .collect(),
                    ),
                    conts,
                )
            }
        }
    }

    fn apply_inst(&self, inst: Instruction) -> Instruction {
        match inst {
            Instruction::Crash => Instruction::Crash,
            Instruction::Reserve(res) => Instruction::Reserve(res),
            Instruction::Copy(target, value) => {
                let target = self.apply_target(target);
                let value = self.apply_value(value);
                Instruction::Copy(target, value)
            }
            Instruction::Index(target, value, index) => {
                let target = self.apply_target(target);
                let value = self.apply_value(value);
                Instruction::Index(target, value, index)
            }
            Instruction::Tuple(target, values) => {
                let target = self.apply_target(target);
                let values = values
                    .into_iter()
                    .map(|value| self.apply_value(value))
                    .collect();
                Instruction::Tuple(target, values)
            }
        }
    }

    fn apply_reg(&self, reg: Register) -> Register {
        println!("applying reg {reg:?}");
        match reg {
            Register::Virtual(reg) => self.allocation.mapping.get(&reg.id).copied().unwrap(),
            _ => reg,
        }
    }

    fn apply_value(&self, value: Value) -> Value {
        match value {
            Value::Integer(i) => Value::Integer(i),
            Value::Name(name) => Value::Name(name),
            Value::Register(reg) => Value::Register(self.apply_reg(reg)),
        }
    }

    fn apply_target(&self, target: Target) -> Target {
        match target {
            Target::Name(name) => Target::Name(name),
            Target::Register(reg) => Target::Register(self.apply_reg(reg)),
        }
    }

    fn map_block(&self, block: BlockId) -> BlockId {
        self.block_map.get(&block).copied().unwrap()
    }
}
