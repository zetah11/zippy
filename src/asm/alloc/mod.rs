mod allocation;
mod info;
mod interfere;
mod live;
mod priority;

#[derive(Debug)]
pub struct Constraints {
    pub max_physical: usize,
}

use crate::lir::{
    BlockId, Branch, Instruction, ProcBuilder, Procedure, Register, Target, Types, Value,
};
use allocation::{allocate, Allocation};

pub fn regalloc(constraints: &Constraints, types: &Types, proc: Procedure) -> Procedure {
    let allocation = allocate(types, &proc, constraints);
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

    pub fn apply(&self, mut proc: Procedure) -> Procedure {
        let mut builder =
            ProcBuilder::new(self.apply_reg(proc.param), proc.continuations.drain(..));

        let mut worklist = vec![proc.entry];

        let mut new_id = builder.fresh_id();
        let entry = new_id;
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

            builder.add(
                new_id,
                block.param.map(|reg| self.apply_reg(reg)),
                insts,
                branch,
            );

            if proc.exits.contains(&id) {
                exits.push(new_id);
            }

            new_id = builder.fresh_id();
        }

        builder.build(entry, exits)
    }

    fn apply_branch(&self, branch: Branch) -> (Branch, Vec<BlockId>) {
        match branch {
            Branch::Return(cont, value) => (Branch::Return(cont, self.apply_value(value)), vec![]),
            Branch::Jump(succ, arg) => (
                Branch::Jump(succ, arg.map(|value| self.apply_value(value))),
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
                    then: (then, then_arg),
                    elze: (elze, elze_arg),
                };

                (res, vec![then, elze])
            }
            Branch::Call(fun, arg, conts) => {
                let fun = self.apply_value(fun);
                let arg = self.apply_value(arg);
                (Branch::Call(fun, arg, conts.clone()), conts)
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
        match reg {
            Register::Virtual { reg, ndx: _ } => self.allocation.map.get(&reg.id).copied().unwrap(),
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
}
