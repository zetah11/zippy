use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::info::ProcInfo;
use crate::lir::{BlockId, Branch, Instruction, Procedure, Register, Target, Value};

pub fn liveness(info: &ProcInfo, procedure: &Procedure) -> HashMap<Register, HashSet<Position>> {
    let mut analyzer = Analyzer::new(info, procedure);
    analyzer.analyze();
    analyzer.live_at
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Time {
    Load,
    Store,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Position {
    /// The entry of the function, i.e. "before" the entry block. Applies to e.g. function parameters.
    Entry,

    /// The entry of a block, immediately "before" any of its instruction. Applies to e.g. block parameters.
    Parameter(BlockId),

    /// Immediately after an instruction in a block.
    Instruction(BlockId, usize, Time),

    /// Immediately after the branch of a block.
    Branch(BlockId),
}

#[derive(Debug)]
struct Analyzer<'a> {
    live_at: HashMap<Register, HashSet<Position>>,
    live_in: HashMap<BlockId, HashSet<Register>>,

    info: &'a ProcInfo,
    procedure: &'a Procedure,
}

impl<'a> Analyzer<'a> {
    pub fn new(info: &'a ProcInfo, procedure: &'a Procedure) -> Self {
        Self {
            live_at: HashMap::new(),
            live_in: HashMap::new(),
            info,
            procedure,
        }
    }

    pub fn analyze(&mut self) {
        for param in self.procedure.params.iter() {
            self.live_at
                .entry(*param)
                .or_default()
                .insert(Position::Entry);
        }

        let mut worklist = self.make_worklist();
        while let Some(id) = worklist.pop() {
            let next = self.analyze_block(id);
            worklist.extend(next);
        }
    }

    fn analyze_block(&mut self, id: BlockId) -> Vec<BlockId> {
        let mut live = self.out_for(&id);
        self.register(&live, Position::Branch(id));

        let block = self.procedure.get(&id).clone();
        let branch = self.procedure.get_branch(block.branch);

        live.extend(self.analyze_branch(branch));
        self.register(&live, Position::Branch(id));

        for inst in block.insts.rev() {
            let instruction = self.procedure.get_instruction(inst);
            let (kills, gens) = self.analyze_instruction(instruction);

            self.register(&kills, Position::Instruction(id, inst, Time::Store));
            self.register(&live, Position::Instruction(id, inst, Time::Store));

            for killed in kills {
                live.remove(&killed);
            }

            live.extend(gens);
            self.register(&live, Position::Instruction(id, inst, Time::Load));
        }

        if let Some(param) = block.param {
            live.insert(param);
            self.register(&live, Position::Parameter(id));
            live.remove(&param);
        }

        let block = self.live_in.entry(id).or_default();
        let old = block.len();
        block.extend(live);

        match block.len().cmp(&old) {
            Ordering::Equal => vec![],
            Ordering::Greater => self.info.preds(&id).collect(),
            Ordering::Less => unreachable!(),
        }
    }

    fn analyze_branch(&self, branch: &Branch) -> HashSet<Register> {
        match branch {
            Branch::Call(fun, args, _) => self
                .reg_in_value(fun)
                .into_iter()
                .chain(args.iter().copied())
                .collect(),

            Branch::Jump(_, arg) => arg
                .as_ref()
                .and_then(|arg| self.reg_in_value(arg))
                .into_iter()
                .collect(),

            Branch::JumpIf {
                left,
                right,
                then: (_, then),
                elze: (_, elze),
                ..
            } => self
                .reg_in_value(left)
                .into_iter()
                .chain(self.reg_in_value(right))
                .chain(then.as_ref().and_then(|arg| self.reg_in_value(arg)))
                .chain(elze.as_ref().and_then(|arg| self.reg_in_value(arg)))
                .collect(),

            Branch::Return(_, arg) => self.reg_in_value(arg).into_iter().collect(),
        }
    }

    fn analyze_instruction(
        &self,
        instruction: &Instruction,
    ) -> (HashSet<Register>, HashSet<Register>) {
        match instruction {
            Instruction::Crash => (HashSet::new(), HashSet::new()),
            Instruction::Reserve(_) => (HashSet::new(), HashSet::new()),
            Instruction::Copy(target, value) => {
                let kills = self.reg_in_target(target).into_iter().collect();
                let gens = self.reg_in_value(value).into_iter().collect();

                (kills, gens)
            }

            Instruction::Index(target, value, _) => {
                let kills = self.reg_in_target(target).into_iter().collect();
                let gens = self.reg_in_value(value).into_iter().collect();

                (kills, gens)
            }

            Instruction::Tuple(target, values) => {
                let kills = self.reg_in_target(target).into_iter().collect();
                let gens = values
                    .iter()
                    .flat_map(|value| self.reg_in_value(value))
                    .collect();

                (kills, gens)
            }
        }
    }

    fn make_worklist(&self) -> Vec<BlockId> {
        let mut worklist = self.procedure.exits.clone();
        let mut res = Vec::new();

        while let Some(id) = worklist.pop() {
            worklist.extend(self.info.preds(&id));
            res.push(id);
        }

        res
    }

    fn reg_in_value(&self, value: &Value) -> Option<Register> {
        match value {
            Value::Register(reg) => Some(*reg),
            _ => None,
        }
    }

    fn reg_in_target(&self, target: &Target) -> Option<Register> {
        match target {
            Target::Register(reg) => Some(*reg),
            _ => None,
        }
    }

    /// Returns `true` if any liveness information changed.
    fn register(&mut self, live: &HashSet<Register>, at: Position) {
        for reg in live.iter() {
            self.live_at.entry(*reg).or_default().insert(at);
        }
    }

    fn out_for(&self, id: &BlockId) -> HashSet<Register> {
        let mut res = HashSet::new();
        for succ in self.info.succs(id) {
            res.extend(self.live_in.get(&succ).into_iter().flatten().copied());
        }
        res
    }
}
