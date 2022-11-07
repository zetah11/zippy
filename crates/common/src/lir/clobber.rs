use std::collections::HashSet;

use super::{BlockId, Branch, Instruction, Procedure, Register, Target, TargetNode};

/// Compute a set of all the physical registers which this procedure clobbers (i.e. assigns or overwrites).
pub fn clobbered(proc: &Procedure) -> HashSet<usize> {
    let mut clobberer = Clobberer::default();
    clobberer.clobber_proc(proc);
    clobberer.regs
}

#[derive(Debug, Default)]
struct Clobberer {
    regs: HashSet<usize>,
    worklist: Vec<BlockId>,
}

impl Clobberer {
    pub fn clobber_proc(&mut self, proc: &Procedure) {
        self.worklist.push(proc.entry);

        while let Some(block) = self.worklist.pop() {
            if !proc.has_block(&block) {
                continue;
            }

            let block = proc.get(&block);

            for inst in block.insts.clone() {
                let inst = proc.get_instruction(inst);
                self.clobber_inst(inst);
            }

            self.clobber_branch(proc.get_branch(block.branch));
        }
    }

    fn clobber_branch(&mut self, branch: &Branch) {
        match branch {
            Branch::Return(..) => {}

            Branch::Jump(to, _) => {
                self.worklist.push(*to);
            }

            Branch::JumpIf { then, elze, .. } => {
                self.worklist.extend([then, elze]);
            }

            Branch::Call(.., conts) => {
                self.worklist.extend(conts.iter().copied());
            }
        }
    }

    fn clobber_inst(&mut self, inst: &Instruction) {
        match inst {
            Instruction::Crash => {}
            Instruction::Copy(target, _) => self.clobber_target(target),
            Instruction::Index(target, ..) => self.clobber_target(target),
            Instruction::Tuple(target, ..) => self.clobber_target(target),
        }
    }

    fn clobber_target(&mut self, target: &Target) {
        match &target.node {
            TargetNode::Register(reg) => self.clobber_reg(reg),
            TargetNode::Name(_) => {}
        }
    }

    fn clobber_reg(&mut self, reg: &Register) {
        match reg {
            Register::Physical(reg) => {
                self.regs.insert(*reg);
            }
            Register::Frame(..) => {}
            Register::Virtual(_) => {}
        }
    }
}
