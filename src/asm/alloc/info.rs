use std::collections::{HashMap, HashSet};

use crate::lir::{BlockId, Branch, Inst, Proc, Register, Target, Value};

#[derive(Debug)]
pub struct ProcInfo {
    pub preds: HashMap<BlockId, Vec<BlockId>>,
    pub succs: HashMap<BlockId, Vec<BlockId>>,
    pub kills: HashMap<BlockId, HashSet<Register>>,
    pub gens: HashMap<BlockId, HashSet<Register>>,
}

impl ProcInfo {
    pub fn preds(&self, block: &BlockId) -> impl Iterator<Item = BlockId> + '_ {
        self.preds.get(block).into_iter().flatten().copied()
    }

    pub fn succs(&self, block: &BlockId) -> impl Iterator<Item = BlockId> + '_ {
        self.succs.get(block).into_iter().flatten().copied()
    }

    pub fn kills(&self, block: &BlockId) -> &HashSet<Register> {
        self.kills.get(block).unwrap()
    }

    pub fn gens(&self, block: &BlockId) -> &HashSet<Register> {
        self.gens.get(block).unwrap()
    }
}

pub fn info(proc: &Proc) -> ProcInfo {
    fn value_to_reg(value: &Value) -> Option<Register> {
        if let Value::Register(reg) = value {
            Some(*reg)
        } else {
            None
        }
    }

    fn target_to_reg(target: &Target) -> Option<Register> {
        if let Target::Register(reg) = target {
            Some(*reg)
        } else {
            None
        }
    }

    let mut kills: HashMap<BlockId, HashSet<Register>> = HashMap::new();
    let mut gens: HashMap<BlockId, HashSet<Register>> = HashMap::new();

    let mut worklist = vec![proc.entry];
    let mut edges = Vec::new();

    while let Some(from) = worklist.pop() {
        let block = proc.get(&from);
        let gens = gens.entry(from).or_default();
        let kills = kills.entry(from).or_default();

        match &block.branch {
            Branch::Jump(to) => {
                edges.push((from, *to));
                worklist.push(*to);
            }

            Branch::JumpIf {
                left,
                right,
                consequence,
                alternative,
                conditional: _,
            } => {
                edges.push((from, *consequence));
                edges.push((from, *alternative));

                worklist.push(*consequence);
                worklist.push(*alternative);

                gens.extend(value_to_reg(left));
                gens.extend(value_to_reg(right));
            }

            Branch::Return(value) => {
                gens.extend(value_to_reg(value));
            }
        }

        for inst in block.insts.iter() {
            match inst {
                Inst::Move(target, value) => {
                    gens.extend(value_to_reg(value));
                    kills.extend(target_to_reg(target));
                }

                Inst::Push(value) => {
                    gens.extend(value_to_reg(value));
                }

                Inst::Pop(target) => {
                    kills.extend(target_to_reg(target));
                }

                Inst::Call(value) => {
                    gens.extend(value_to_reg(value));
                }
            }
        }
    }

    let mut preds: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    let mut succs: HashMap<BlockId, Vec<BlockId>> = HashMap::new();

    for (from, to) in edges {
        preds.entry(to).or_default().push(from);
        succs.entry(from).or_default().push(to);
    }

    ProcInfo {
        preds,
        succs,
        kills,
        gens,
    }
}
