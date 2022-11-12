use std::cmp::Ordering;

use common::lir::{
    BaseOffset, BlockId, Branch, Condition, Instruction, Procedure, Register, TargetNode, ValueNode,
};
use iced_x86::code_asm::{rbx, rsp};

use super::instruction::Operand;
use super::{Constraints, Lowerer};
use crate::asm::AllocConstraints;

impl Lowerer<'_> {
    pub fn lower_block(&mut self, order: &[BlockId], procedure: &Procedure, block: BlockId) {
        self.set_block_label(block);
        let block = procedure.get(&block);

        for inst in block.insts.clone() {
            let inst = procedure.get_instruction(inst);
            self.lower_instruction(procedure, inst);
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
                let fun = self.value_operand(procedure, fun).unwrap();
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
                args: _args,
                then,
                elze,
            } => {
                let left = self.value_operand(procedure, left).unwrap();
                let right = self.value_operand(procedure, right).unwrap();

                // TODO: swap things around if this is invalid (e.g. integer on the left side)
                self.asm_cmp(left, right);

                let then_follows = order.first().map(|next| then == next).unwrap_or(false);
                let elze_follows = order.first().map(|next| elze == next).unwrap_or(false);

                assert!(!(then_follows && elze_follows));

                match cond {
                    Condition::Equal if elze_follows => self.asm_je(Operand::Block(*then)),
                    Condition::Equal if then_follows => self.asm_jne(Operand::Block(*elze)),
                    Condition::Equal => {
                        self.asm_je(Operand::Block(*then));
                        self.asm_jmp(Operand::Block(*elze));
                    }

                    Condition::Greater if elze_follows => self.asm_jg(Operand::Block(*then)),
                    Condition::Greater if then_follows => self.asm_jle(Operand::Block(*elze)),
                    Condition::Greater => {
                        self.asm_jg(Operand::Block(*then));
                        self.asm_jmp(Operand::Block(*elze));
                    }

                    Condition::Less if elze_follows => self.asm_jl(Operand::Block(*then)),
                    Condition::Less if then_follows => self.asm_jge(Operand::Block(*elze)),
                    Condition::Less => {
                        self.asm_jl(Operand::Block(*then));
                        self.asm_jmp(Operand::Block(*elze));
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

    fn lower_instruction(&mut self, procedure: &Procedure, inst: &Instruction) {
        match inst {
            Instruction::Copy(target, value) => {
                if let TargetNode::Register(Register::Frame(
                    BaseOffset::Argument { offset, total },
                    ty,
                )) = target.node
                {
                    let offset = total - offset;
                    let size = Constraints::sizeof(&self.program.types, &ty);

                    match self.arg_offset.cmp(&offset) {
                        Ordering::Equal => {
                            let value = self.value_operand(procedure, value).unwrap();
                            self.asm_push(value);
                            self.arg_offset += size;
                            return;
                        }

                        Ordering::Less => {
                            let value = self.value_operand(procedure, value).unwrap();
                            let diff = offset - self.arg_offset;
                            self.asm_push(value);
                            self.asm.sub(rsp, diff as i32).unwrap();
                            self.arg_offset = offset + size;
                            return;
                        }

                        Ordering::Greater => {}
                    }
                }

                if let ValueNode::Register(Register::Frame(
                    BaseOffset::Argument { offset, total },
                    ty,
                )) = value.node
                {
                    let offset = total - offset;
                    let size = Constraints::sizeof(&self.program.types, &ty);

                    match self.arg_offset.cmp(&offset) {
                        Ordering::Equal => {
                            let target = self.target_operand(procedure, target).unwrap();
                            self.asm_pop(target);
                            self.arg_offset -= size;
                            return;
                        }

                        Ordering::Greater => {
                            let target = self.target_operand(procedure, target).unwrap();
                            let diff = self.arg_offset - offset;
                            self.asm_pop(target);
                            self.asm.add(rsp, (diff - size) as i32).unwrap();
                            self.arg_offset = offset;
                            return;
                        }

                        Ordering::Less => {}
                    }
                }

                let target = self.target_operand(procedure, target).unwrap();
                let value = self.value_operand(procedure, value).unwrap();

                if target == value {
                    return;
                }

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
