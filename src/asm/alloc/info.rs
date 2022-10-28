use std::collections::{HashMap, HashSet};

use crate::lir::{BlockId, Branch, Procedure, Register};

#[derive(Debug)]
pub struct ProcInfo {
    pub preds: HashMap<BlockId, Vec<BlockId>>,
    pub succs: HashMap<BlockId, Vec<BlockId>>,

    pub args: HashSet<Register>,
}

impl ProcInfo {
    pub fn preds(&self, block: &BlockId) -> impl Iterator<Item = BlockId> + '_ {
        self.preds.get(block).into_iter().flatten().copied()
    }

    pub fn succs(&self, block: &BlockId) -> impl Iterator<Item = BlockId> + '_ {
        self.succs.get(block).into_iter().flatten().copied()
    }
}

pub fn info(proc: &Procedure) -> ProcInfo {
    let mut args: HashSet<Register> = HashSet::new();
    args.extend(proc.params.iter().copied());

    let mut worklist = vec![proc.entry];
    let mut edges = Vec::new();

    while let Some(from) = worklist.pop() {
        let block = proc.get(&from);

        args.extend(block.param.iter().copied());

        match proc.get_branch(block.branch) {
            Branch::Jump(to, ..) => {
                edges.push((from, *to));
                worklist.push(*to);
            }

            Branch::JumpIf {
                then: (then, _),
                elze: (elze, _),
                ..
            } => {
                edges.push((from, *then));
                edges.push((from, *elze));

                worklist.push(*then);
                worklist.push(*elze);
            }

            Branch::Return(_, regs) => {
                args.extend(regs.iter().copied());
            }

            Branch::Call(_, call_args, conts) => {
                let call_args = call_args
                    .iter()
                    .filter(|reg| matches!(reg, Register::Virtual(..)));

                args.extend(call_args);

                // skip any tail calls
                edges.extend(
                    conts
                        .iter()
                        .filter(|block| proc.has_block(block))
                        .map(|block| (from, *block)),
                );
                worklist.extend(conts.iter().filter(|block| proc.has_block(block)));
            }
        }
    }

    let mut preds: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    let mut succs: HashMap<BlockId, Vec<BlockId>> = HashMap::new();

    for (from, to) in edges {
        preds.entry(to).or_default().push(from);
        succs.entry(from).or_default().push(to);
    }

    ProcInfo { preds, succs, args }
}
