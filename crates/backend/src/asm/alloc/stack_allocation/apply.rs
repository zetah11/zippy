use std::collections::HashMap;

use common::lir::{
    BaseOffset, BlockId, Branch, Instruction, ProcBuilder, Procedure, Register, Target, TargetNode,
    Value, ValueNode,
};

use super::{Allocation, Place};

pub(super) fn apply(allocation: Allocation, procedure: Procedure) -> Procedure {
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
                block
                    .params
                    .iter()
                    .copied()
                    .map(|reg| self.apply_reg(reg))
                    .collect(),
                insts,
                branch,
            );

            if proc.exits.contains(&id) {
                exits.push(new_id);
            }
        }

        builder.frame_space(self.allocation.frame_space);
        builder.build(entry, exits)
    }

    fn apply_branch(&self, branch: Branch) -> (Branch, Vec<BlockId>) {
        match branch {
            Branch::Return(cont, values) => (
                Branch::Return(
                    self.map_block(cont),
                    values
                        .into_iter()
                        .map(|(reg, ty)| (self.apply_reg(reg), ty))
                        .collect(),
                ),
                vec![],
            ),
            Branch::Jump(succ, args) => (
                Branch::Jump(
                    self.map_block(succ),
                    args.into_iter().map(|reg| self.apply_reg(reg)).collect(),
                ),
                vec![succ],
            ),
            Branch::JumpIf {
                left,
                cond,
                right,
                args,
                then,
                elze,
            } => {
                let left = self.apply_value(left);
                let right = self.apply_value(right);
                let args = args.into_iter().map(|reg| self.apply_reg(reg)).collect();
                let res = Branch::JumpIf {
                    left,
                    cond,
                    right,
                    args,
                    then: self.map_block(then),
                    elze: self.map_block(elze),
                };

                (res, vec![then, elze])
            }
            Branch::Call(fun, args, conts) => {
                let fun = self.apply_value(fun);
                let args = args
                    .into_iter()
                    .map(|(arg, ty)| (self.apply_reg(arg), ty))
                    .collect();
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
        match reg {
            Register::Virtual(reg) => {
                let (offset, _) = self.allocation.mapping.get(&reg.id).copied().unwrap();
                let offset = match offset {
                    Place::Argument { offset, total } => BaseOffset::Argument { offset, total },
                    Place::Parameter { offset, total } => BaseOffset::Parameter { offset, total },
                    Place::Local(offset) => BaseOffset::Local(offset),
                };

                Register::Frame(offset, reg.ty)
            }
            _ => reg,
        }
    }

    fn apply_value(&self, value: Value) -> Value {
        let node = match value.node {
            ValueNode::Integer(i) => ValueNode::Integer(i),
            ValueNode::Name(name) => ValueNode::Name(name),
            ValueNode::Register(reg) => ValueNode::Register(self.apply_reg(reg)),
        };

        Value {
            node,
            ty: value.ty,
            span: value.span,
        }
    }

    fn apply_target(&self, target: Target) -> Target {
        let node = match target.node {
            TargetNode::Name(name) => TargetNode::Name(name),
            TargetNode::Register(reg) => TargetNode::Register(self.apply_reg(reg)),
        };

        Target {
            node,
            ty: target.ty,
            span: target.span,
        }
    }

    fn map_block(&self, block: BlockId) -> BlockId {
        self.block_map.get(&block).copied().unwrap()
    }
}
