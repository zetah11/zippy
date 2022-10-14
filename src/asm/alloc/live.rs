use std::collections::{HashMap, HashSet};

use super::info::{info, ProcInfo};
use crate::lir::{BlockId, Proc, Register};

pub fn liveness(proc: &Proc) -> LivenessFacts {
    let info = info(proc);
    let mut analyzer = LivenessAnalyzer::new(proc, &info);
    analyzer.iterate();

    LivenessFacts {
        live_in: analyzer.in_facts,
        live_out: analyzer.out_facts,
    }
}

#[derive(Debug)]
pub struct LivenessFacts {
    pub live_in: HashMap<BlockId, HashSet<Register>>,
    pub live_out: HashMap<BlockId, HashSet<Register>>,
}

struct LivenessAnalyzer<'a> {
    in_facts: HashMap<BlockId, HashSet<Register>>,
    out_facts: HashMap<BlockId, HashSet<Register>>,
    proc: &'a Proc,
    info: &'a ProcInfo,

    worklist: Vec<BlockId>,
}

impl<'a> LivenessAnalyzer<'a> {
    pub fn new(proc: &'a Proc, info: &'a ProcInfo) -> Self {
        let mut res = Self {
            in_facts: HashMap::new(),
            out_facts: HashMap::new(),
            proc,
            info,

            worklist: Vec::new(),
        };

        res.init_worklist();

        res
    }

    pub fn iterate(&mut self) {
        while let Some(block) = self.worklist.pop() {
            let out = self.compute_out(&block);
            let inb = self.compute_in(&out, &block);

            let in_fact = self.in_facts.entry(block).or_default();
            let out_fact = self.out_facts.entry(block).or_default();

            if &out != out_fact || &inb != out_fact {
                *out_fact = out;
                *in_fact = inb;

                self.worklist.extend(self.info.preds(&block));
            }
        }
    }

    fn init_worklist(&mut self) {
        let mut worklist = vec![self.proc.exit];

        while let Some(block) = worklist.pop() {
            if self.worklist.contains(&block) {
                continue;
            }

            worklist.extend(self.info.preds(&block));
            self.worklist.push(block);
        }
    }

    /// ```text
    /// in(b) = union(out(b) - kill(b), gen(b))
    /// ```
    fn compute_in(&self, out: &HashSet<Register>, block: &BlockId) -> HashSet<Register> {
        let without_kill = out.difference(self.info.kills(block));

        let mut res = self.info.gens(block).clone();
        res.extend(without_kill);

        res
    }

    /// ```text
    /// out(b) = union(in(s) for s in succ(b))
    /// ```
    fn compute_out(&self, block: &BlockId) -> HashSet<Register> {
        let mut res = HashSet::new();
        for succ in self.info.succs(block) {
            res.extend(self.in_facts.get(&succ).into_iter().flatten().copied());
        }
        res
    }
}
