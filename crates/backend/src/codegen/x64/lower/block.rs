use super::{instruction::insert_move, lir, x64, Lowerer};
use crate::codegen::x64::regid_to_reg;

impl Lowerer<'_> {
    pub fn lower_block(
        &mut self,
        order: &[lir::BlockId],
        id: lir::BlockId,
        proc: &lir::Procedure,
        block: lir::Block,
    ) -> x64::Block {
        let mut insts = Vec::new();

        // prologue
        if id == proc.entry && !proc.frame_space.map(|space| space == 0).unwrap_or(false) {
            insts.extend([
                x64::Instruction::Push(x64::Operand::Register(x64::Register::Rbp)),
                x64::Instruction::Mov(
                    x64::Operand::Register(x64::Register::Rbp),
                    x64::Operand::Register(x64::Register::Rsp),
                ),
            ]);
        }

        for inst in block.insts {
            let inst = proc.get_instruction(inst);
            match inst.clone() {
                lir::Instruction::Crash => todo!(),
                lir::Instruction::Copy(target, value) => {
                    let dest = self.lower_target(target);
                    let src = self.lower_value(value);
                    insert_move(&mut insts, dest, src);
                }

                lir::Instruction::Index(name, lir::Value::Name(of), at) => {
                    let of = self.lower_name(of);
                    let name = self.lower_target(name);

                    insts.push(x64::Instruction::Mov(
                        x64::Operand::Register(x64::Register::Rax),
                        x64::Operand::Location(of),
                    ));

                    insts.push(x64::Instruction::Mov(
                        name,
                        x64::Operand::Memory(x64::Address {
                            reg: Some(x64::Register::Rax),
                            offset: None,
                            scale: x64::Scale::One,
                            displacement: Some(i32::try_from(at).unwrap()),
                        }),
                    ));
                }

                lir::Instruction::Index(name, lir::Value::Register(reg), at) => {
                    let name = self.lower_target(name);
                    let value = match reg {
                        lir::Register::Physical(reg) => x64::Operand::Memory(x64::Address {
                            reg: Some(regid_to_reg(reg)),
                            offset: None,
                            scale: x64::Scale::One,
                            displacement: Some(i32::try_from(at).unwrap()),
                        }),

                        lir::Register::Frame(offset, _) => x64::Operand::Memory(x64::Address {
                            reg: Some(x64::Register::Rbp),
                            offset: None,
                            scale: x64::Scale::One,
                            displacement: Some(i32::try_from(offset + at as isize).unwrap()),
                        }),

                        lir::Register::Virtual(_) => unreachable!(),
                    };

                    if matches!(name, x64::Operand::Memory(_)) {
                        insts.push(x64::Instruction::Mov(
                            x64::Operand::Register(x64::Register::Rax),
                            value,
                        ));

                        insts.push(x64::Instruction::Mov(
                            name,
                            x64::Operand::Register(x64::Register::Rax),
                        ))
                    } else {
                        insert_move(&mut insts, name, value);
                    }
                }

                lir::Instruction::Index(_, lir::Value::Integer(_), _) => unreachable!(),

                lir::Instruction::Reserve(v) => {
                    insts.push(x64::Instruction::Sub(
                        x64::Operand::Register(x64::Register::Rsp),
                        x64::Operand::Immediate(x64::Immediate::Imm64(u64::try_from(v).unwrap())),
                    ));
                }

                lir::Instruction::Tuple(..) => todo!(),
            }
        }

        match proc.get_branch(block.branch) {
            lir::Branch::Jump(to, values) => {
                for value in values {
                    let value = self.lower_value(value.clone());
                    insts.push(x64::Instruction::Mov(
                        x64::Operand::Register(x64::Register::Rax),
                        value,
                    ));
                }

                let to = self.blocks.get(to).unwrap();
                insts.push(x64::Instruction::Jump(x64::Operand::Location(*to)));
            }

            lir::Branch::Return(cont, _values) => {
                let index = proc.continuations.iter().position(|id| id == cont).unwrap();

                if index != 0 {
                    // todo: horrid!
                    for _ in 0..index {
                        insts.push(x64::Instruction::Pop(x64::Operand::Register(
                            x64::Register::Rax,
                        )));
                    }
                }

                if !proc.frame_space.map(|space| space == 0).unwrap_or(false) {
                    insts.push(x64::Instruction::Leave);
                }

                insts.push(x64::Instruction::Ret);
            }

            lir::Branch::Call(fun, _args, conts) => {
                // CONTINUATIONS!
                // okay so a convention is that the return continuation is always the first continuation passed.
                // additionally, here in x64 land, we can say that continuations are passed "right to left" so to speak;
                // the last continuation is passed first, which places the return continuation on the top.
                // thus, if the return continuation is *also* the next block we serialize, a call becomes a simple call
                // since that takes care of the return continuation for us. nice :)

                let mut retc: Option<x64::Name> = None;
                for cont in conts.iter().rev() {
                    if let Some(cont) = retc.take() {
                        insts.extend([
                            x64::Instruction::Mov(
                                x64::Operand::Register(x64::Register::Rax),
                                x64::Operand::Location(cont),
                            ),
                            x64::Instruction::Push(x64::Operand::Register(x64::Register::Rax)),
                        ]);
                    }

                    retc = Some(
                        self.blocks
                            .get(cont)
                            .copied()
                            .expect("tail calls unimplemented!"),
                    );
                }

                let call = if let Some(retc) = retc {
                    let this_block = order.iter().position(|block| block == &id);

                    match this_block {
                        Some(pos) if pos + 1 < order.len() => {
                            let next = self.blocks.get(&order[pos + 1]).unwrap();
                            let call = next == &retc;
                            if call {
                                true
                            } else {
                                insts.extend([
                                    x64::Instruction::Mov(
                                        x64::Operand::Register(x64::Register::Rax),
                                        x64::Operand::Location(retc),
                                    ),
                                    x64::Instruction::Push(x64::Operand::Register(
                                        x64::Register::Rax,
                                    )),
                                ]);
                                false
                            }
                        }

                        _ => false,
                    }
                } else {
                    false
                };

                let fun = self.lower_value(fun.clone());

                if call {
                    insts.push(x64::Instruction::Call(fun));
                } else {
                    insts.push(x64::Instruction::Jump(fun));
                }
            }

            lir::Branch::JumpIf { .. } => todo!(),
        }

        x64::Block { insts }
    }
}
