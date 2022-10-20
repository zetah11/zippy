use std::collections::{HashMap, HashSet};

use crate::lir::{BlockId, Branch, Instruction, Procedure, Register, Target, Value};

#[derive(Debug)]
pub struct ProcInfo {
    pub preds: HashMap<BlockId, Vec<BlockId>>,
    pub succs: HashMap<BlockId, Vec<BlockId>>,
    pub kills: HashMap<BlockId, HashSet<Register>>,
    pub uses: HashMap<BlockId, HashSet<Register>>,
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

    pub fn uses(&self, block: &BlockId) -> &HashSet<Register> {
        self.uses.get(block).unwrap()
    }
}

pub fn info(proc: &Procedure) -> ProcInfo {
    fn value_to_reg(value: &Value) -> Option<Register> {
        if let Value::Register(reg) = value {
            if matches!(reg, Register::Virtual { .. }) {
                Some(*reg)
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }

    fn target_to_reg(target: &Target) -> Option<Register> {
        if let Target::Register(reg) = target {
            if matches!(reg, Register::Virtual { .. }) {
                Some(*reg)
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }

    let mut kills: HashMap<BlockId, HashSet<Register>> = HashMap::new();
    let mut gens: HashMap<BlockId, HashSet<Register>> = HashMap::new();

    gens.entry(proc.entry).or_default().insert(proc.param);

    let mut worklist = vec![proc.entry];
    let mut edges = Vec::new();

    while let Some(from) = worklist.pop() {
        let block = proc.get(&from);
        let gens = gens.entry(from).or_default();
        let kills = kills.entry(from).or_default();

        block.param.map(|reg| gens.insert(reg));

        match proc.get_branch(block.branch) {
            Branch::Jump(to, param) => {
                gens.extend(param.as_ref().and_then(value_to_reg));

                edges.push((from, *to));
                worklist.push(*to);
            }

            Branch::JumpIf {
                left,
                cond: _,
                right,
                then,
                elze,
            } => {
                edges.push((from, then.0));
                edges.push((from, elze.0));

                worklist.push(then.0);
                worklist.push(elze.0);

                gens.extend(value_to_reg(left));
                gens.extend(value_to_reg(right));

                gens.extend(then.1.as_ref().and_then(value_to_reg));
                gens.extend(elze.1.as_ref().and_then(value_to_reg));
            }

            Branch::Return(_, value) => {
                gens.extend(value_to_reg(value));
            }

            Branch::Call(fun, arg, conts) => {
                gens.extend(value_to_reg(fun));
                gens.extend(value_to_reg(arg));

                // skip any tail calls
                worklist.extend(conts.iter().filter(|block| proc.has_block(block)));
            }
        }

        for inst in block.insts.clone() {
            match proc.get_instruction(inst) {
                Instruction::Crash | Instruction::Reserve(_) => {}

                Instruction::Copy(target, value) => {
                    gens.extend(value_to_reg(value));
                    kills.extend(target_to_reg(target));
                }

                Instruction::Tuple(target, values) => {
                    gens.extend(values.iter().flat_map(value_to_reg));
                    kills.extend(target_to_reg(target));
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
        uses: gens,
    }
}
