use std::collections::HashMap;

use super::super::constraint::Constraints;
use super::approximate::LivenessFacts;
use super::range::{
    block_header_range, branch_range, point, procedure_header_range, span, LiveRange, LiveRanges,
};
use crate::lir::{BlockId, Branch, Instruction, Procedure, Register, Target, Value};

pub fn precise_liveness(
    info: &LivenessFacts,
    proc: &Procedure,
    constraints: &Constraints,
) -> Liveness {
    let mut analyzer = Analyzer {
        regs: HashMap::new(),
        proc,
        info,
        constraints,
    };

    analyzer.analyze();

    Liveness {
        regs: analyzer.regs,
    }
}

#[derive(Debug)]
pub struct Liveness {
    pub regs: HashMap<Register, LiveRanges>,
}

#[derive(Debug)]
struct Analyzer<'a> {
    regs: HashMap<Register, LiveRanges>,
    proc: &'a Procedure,
    info: &'a LivenessFacts,
    constraints: &'a Constraints,
}

impl Analyzer<'_> {
    pub fn analyze(&mut self) {
        self.proc.params.iter().for_each(|param| {
            self.regs
                .entry(*param)
                .or_default()
                .insert(procedure_header_range(self.proc));
        });

        let mut worklist = vec![self.proc.entry];
        while let Some(id) = worklist.pop() {
            let nexts = self.analyze_block(&id);
            worklist.extend(nexts);
        }
    }

    fn analyze_block(&mut self, id: &BlockId) -> Vec<BlockId> {
        let block = self.proc.get(id);

        if let Some(param) = block.param.as_ref() {
            self.regs
                .entry(*param)
                .or_default()
                .insert(block_header_range(self.proc, *id));
        }

        let mut ranges: HashMap<Register, LiveRange> = HashMap::new();
        let start = point(block.insts.start);
        let end = point(block.insts.end);

        self.info
            .live_in
            .get(id)
            .into_iter()
            .flatten()
            .for_each(|register| {
                ranges.insert(*register, start);
            });

        self.info
            .live_out
            .get(id)
            .into_iter()
            .flatten()
            .for_each(|register| {
                ranges.entry(*register).and_modify(|range| {
                    *range = span(*range, end);
                });
            });

        for at in block.insts.clone() {
            let instruction = self.proc.get_instruction(at);
            match instruction {
                Instruction::Crash => {}
                Instruction::Reserve(_) => {}
                Instruction::Copy(target, value) => {
                    self.def(&mut ranges, target, at);
                    self.usage(&mut ranges, value, at);
                }
                Instruction::Index(target, value, _) => {
                    self.def(&mut ranges, target, at);
                    self.usage(&mut ranges, value, at);
                }
                Instruction::Tuple(target, values) => {
                    self.def(&mut ranges, target, at);
                    values
                        .iter()
                        .for_each(|value| self.usage(&mut ranges, value, at));
                }
            }
        }

        // Any register that is alive at the out edge must also be alive over the branch.
        let end = branch_range(self.proc, block.branch);
        self.info
            .live_out
            .get(id)
            .into_iter()
            .flatten()
            .for_each(|register| {
                if ranges.contains_key(register) {
                    self.regs.entry(*register).or_default().insert(end);
                }
            });

        for (reg, range) in ranges {
            self.regs.entry(reg).or_default().insert(range);
        }

        match self.proc.get_branch(block.branch) {
            Branch::Call(fun, args, conts) => {
                self.point_usage(fun, end);
                args.iter().for_each(|arg| self.point_usage(arg, end));

                self.constraints.call_clobbers.iter().for_each(|reg| {
                    let reg = Register::Physical(*reg);
                    assert!(self.regs.entry(reg).or_default().insert(end));
                });

                conts
                    .iter()
                    .copied()
                    .filter(|id| self.proc.has_block(id))
                    .collect()
            }

            Branch::Jump(to, arg) => {
                if let Some(arg) = arg.as_ref() {
                    self.point_usage(arg, end)
                }
                vec![*to]
            }

            Branch::JumpIf {
                left,
                right,
                then: (then, then_arg),
                elze: (elze, elze_arg),
                ..
            } => {
                self.point_usage(left, end);
                self.point_usage(right, end);
                if let Some(arg) = then_arg.as_ref() {
                    self.point_usage(arg, end)
                }
                if let Some(arg) = elze_arg.as_ref() {
                    self.point_usage(arg, end)
                }
                vec![*then, *elze]
            }

            Branch::Return(_, arg) => {
                self.point_usage(arg, end);
                vec![]
            }
        }
    }

    fn point_usage(&mut self, value: &Value, at: LiveRange) {
        match value {
            Value::Register(reg) => {
                self.regs.entry(*reg).or_default().insert(at);
            }
            Value::Integer(_) | Value::Name(_) => {}
        }
    }

    fn def(&self, ranges: &mut HashMap<Register, LiveRange>, target: &Target, at: usize) {
        let at = point(at);
        match target {
            Target::Register(reg) => {
                ranges
                    .entry(*reg)
                    .and_modify(|range| *range = span(*range, at))
                    .or_insert(at);
            }
            Target::Name(_) => {}
        }
    }

    fn usage(&self, ranges: &mut HashMap<Register, LiveRange>, value: &Value, at: usize) {
        let at = point(at);
        match value {
            Value::Register(reg) => {
                ranges
                    .entry(*reg)
                    .and_modify(|range| *range = span(*range, at))
                    .or_insert(at);
            }
            Value::Name(_) => {}
            Value::Integer(_) => {}
        }
    }
}
