use common::lir::{BlockId, Branch, Condition, Instruction, Procedure};
use iced_x86::code_asm::rbx;

use super::instruction::Operand;
use super::Lowerer;

impl Lowerer<'_> {
    pub fn lower_block(&mut self, order: &[BlockId], procedure: &Procedure, block: BlockId) {
        self.set_block_label(block);
        let block = procedure.get(&block);

        for inst in block.insts.clone() {
            let inst = procedure.get_instruction(inst);
            self.lower_instruction(inst);
        }

        self.lower_branch(order, procedure, procedure.get_branch(block.branch));
    }

    fn lower_branch(&mut self, order: &[BlockId], procedure: &Procedure, branch: &Branch) {
        match branch {
            Branch::Call(fun, _arg, conts) => {
                // If the return continuation is also the next block, then this can be a simple call instruction.
                // Otherwise, we have to manually push the continuation.
                let call = conts
                    .first()
                    .and_then(|id| order.first().map(|next| (id, next)))
                    .map(|(retc, next)| retc == next)
                    .unwrap_or(false);

                for cont in conts[call.into()..].iter().rev() {
                    self.asm_push(Operand::Block(*cont));
                }

                // We don't have to do anything about `_arg`, since any necessary moves should have been set up before.
                let fun = self.value_operand(fun).unwrap();
                if let Operand::Label(name) = fun {
                    if self.program.info.is_extern(&name) {
                        todo!()
                    }
                }

                if call {
                    self.asm_call(fun);
                } else {
                    self.asm_jmp(fun);
                }
            }

            Branch::Jump(to, _args) => {
                // Still don't have to worry about `_args`.
                self.asm_jmp(Operand::Block(*to));
            }

            Branch::JumpIf {
                left,
                cond,
                right,
                then: (then_cont, _then_args),
                elze: (elze_cont, _elze_args),
            } => {
                let left = self.value_operand(left).unwrap();
                let right = self.value_operand(right).unwrap();

                // TODO: swap things around if this is invalid (e.g. integer on the left side)
                self.asm_cmp(left, right);

                let then_follows = order.first().map(|next| then_cont == next).unwrap_or(false);
                let elze_follows = order.first().map(|next| elze_cont == next).unwrap_or(false);

                assert!(!(then_follows && elze_follows));

                match cond {
                    Condition::Equal if elze_follows => self.asm_je(Operand::Block(*then_cont)),
                    Condition::Equal if then_follows => self.asm_jne(Operand::Block(*elze_cont)),
                    Condition::Equal => {
                        self.asm_je(Operand::Block(*then_cont));
                        self.asm_jmp(Operand::Block(*elze_cont));
                    }

                    Condition::Greater if elze_follows => self.asm_jg(Operand::Block(*then_cont)),
                    Condition::Greater if then_follows => self.asm_jle(Operand::Block(*elze_cont)),
                    Condition::Greater => {
                        self.asm_jg(Operand::Block(*then_cont));
                        self.asm_jmp(Operand::Block(*elze_cont));
                    }

                    Condition::Less if elze_follows => self.asm_jl(Operand::Block(*then_cont)),
                    Condition::Less if then_follows => self.asm_jge(Operand::Block(*elze_cont)),
                    Condition::Less => {
                        self.asm_jl(Operand::Block(*then_cont));
                        self.asm_jmp(Operand::Block(*elze_cont));
                    }
                }
            }

            Branch::Return(to, _args) => {
                let contn = procedure
                    .continuations
                    .iter()
                    .enumerate()
                    .filter_map(|(index, id)| (id == to).then_some(index))
                    .next()
                    .expect("cannot return to non-continuation");

                self.asm_leave();

                for _ in 0..contn {
                    self.asm_pop(Operand::Gpr64(rbx));
                }

                let remaining = procedure.continuations.len() - contn;
                if remaining > 1 {
                    self.asm_ret1(u32::try_from((remaining - 1) * 8).unwrap());
                } else {
                    self.asm_ret();
                }
            }
        }
    }

    fn lower_instruction(&mut self, inst: &Instruction) {
        match inst {
            Instruction::Copy(target, value) => {
                let target = self.target_operand(target).unwrap();
                let value = self.value_operand(value).unwrap();

                if target == value {
                    return;
                }

                println!("{target:?} {value:?}");
                self.asm_mov(target, value);
            }

            Instruction::Crash => {
                self.asm.ud2().unwrap();
            }

            Instruction::Index(..) => unreachable!(),
            Instruction::Tuple(..) => unreachable!(),
        }
    }
}
