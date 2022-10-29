use super::super::repr::{Address, Immediate, Instruction, Operand, Register};
use super::register::Rex;
use super::{Encoder, Relocation, RelocationKind};

impl Encoder {
    pub fn encode_instruction(&mut self, inst: Instruction) {
        match inst {
            Instruction::Call(operand) => self.inst_call(operand),
            Instruction::Cmp(left, right) => self.inst_cmp(left, right),
            Instruction::Jump(operand) => self.inst_jmp(operand),
            Instruction::Leave => self.inst_leave(),
            Instruction::Mov(dest, src) => self.inst_mov(dest, src),
            Instruction::Push(operand) => self.inst_push(operand),
            Instruction::Pop(operand) => self.inst_pop(operand),
            Instruction::Ret => self.inst_ret(),
            Instruction::Sub(dest, src) => self.inst_sub(dest, src),
            Instruction::Syscall => self.inst_syscall(),
        }
    }

    fn two_regs(&mut self, opcode: u8, left: Register, right: Register) {
        let (base, left) = self.encode_reg(left);
        let (reg, right) = self.encode_reg(right);
        let rex = Rex::WIDE | Rex::from_reg(reg) | Rex::from_base(base);

        self.code.extend(u8::try_from(rex));
        self.code.extend([opcode, 0xc0 | right << 3 | left]);
    }

    fn reg_mem(&mut self, opcode: u8, left: Register, right: Address) {
        let (reg, left) = self.encode_reg(left);
        let (mem, rm, rest) = self.encode_addr(right);
        let rex = Rex::WIDE | mem | Rex::from_reg(reg);

        self.code.extend(u8::try_from(rex));
        self.code.extend([opcode, left << 3 | rm]);
        self.code.extend(rest);
    }

    fn inst_call(&mut self, operand: Operand) {
        match operand {
            Operand::Location(name) => {
                let at = self.code.len() + 1;
                self.code.extend([0xe8, 0, 0, 0, 0]);

                self.relocations.entry(name).or_default().push(Relocation {
                    kind: RelocationKind::RelativeNext,
                    at,
                });
            }

            Operand::Register(reg) => {
                let (ex, bits) = self.encode_reg(reg);

                self.code.extend(u8::try_from(Rex::from_base(ex)));
                self.code.extend([0xff, 0xd0 | bits]);
            }

            Operand::Memory(addr) => {
                let (rex, rm, rest) = self.encode_addr(addr);
                self.code.extend(u8::try_from(rex));
                self.code.extend([0xff, rm]);
                self.code.extend(rest);
            }

            _ => panic!("invalid call operand"),
        }
    }

    fn inst_cmp(&mut self, left: Operand, right: Operand) {
        match (left, right) {
            (Operand::Register(left), Operand::Register(right)) => {
                self.two_regs(0x3b, left, right);
            }

            (Operand::Register(reg), Operand::Memory(addr)) => {
                self.reg_mem(0x3b, reg, addr);
            }

            (Operand::Memory(addr), Operand::Register(reg)) => {
                self.reg_mem(0x39, reg, addr);
            }

            _ => unimplemented!(),
        }
    }

    fn inst_jmp(&mut self, operand: Operand) {
        match operand {
            Operand::Location(name) => {
                let at = self.code.len() + 1;
                self.code.extend([0xe9, 0, 0, 0, 0]);

                self.relocations.entry(name).or_default().push(Relocation {
                    kind: RelocationKind::Relative,
                    at,
                });
            }

            Operand::Register(reg) => {
                let (base, reg) = self.encode_reg(reg);
                let rex = Rex::from_base(base);

                self.code.extend(u8::try_from(rex));
                self.code.extend([0xff, 0xe0 | reg]);
            }

            Operand::Memory(addr) => {
                let (rex, rm, rest) = self.encode_addr(addr);
                self.code.extend(u8::try_from(rex));
                self.code.extend([0xff, 0x20 | rm]);
                self.code.extend(rest);
            }

            _ => unimplemented!(),
        }
    }

    fn inst_leave(&mut self) {
        self.code.push(0xc9);
    }

    fn inst_mov(&mut self, dest: Operand, src: Operand) {
        match (dest, src) {
            (Operand::Register(dest), Operand::Register(src)) => {
                self.two_regs(0x89, dest, src);
            }

            (Operand::Register(reg), Operand::Memory(addr)) => {
                self.reg_mem(0x8b, reg, addr);
            }

            (Operand::Memory(addr), Operand::Register(reg)) => {
                self.reg_mem(0x89, reg, addr);
            }

            (Operand::Register(reg), Operand::Immediate(value)) => {
                let (wide, rest): (_, Vec<_>) = match value {
                    Immediate::Imm8(value) => (Rex::NONE, (value as u32).to_le_bytes().into()),
                    Immediate::Imm16(value) => (Rex::NONE, (value as u32).to_le_bytes().into()),
                    Immediate::Imm32(value) => (Rex::NONE, value.to_le_bytes().into()),
                    Immediate::Imm64(value) => (Rex::WIDE, value.to_le_bytes().into()),
                };

                let (base, reg) = self.encode_reg(reg);
                let rex = wide | Rex::from_base(base);

                self.code.extend(u8::try_from(rex));
                self.code.push(0xb8 | reg);
                self.code.extend(rest);
            }

            (Operand::Memory(addr), Operand::Immediate(value)) => {
                let (wide, imm) = match value {
                    Immediate::Imm8(value) => (Rex::NONE, (value as u32).to_le_bytes()),
                    Immediate::Imm16(value) => (Rex::NONE, (value as u32).to_le_bytes()),
                    Immediate::Imm32(value) => (Rex::NONE, value.to_le_bytes()),
                    Immediate::Imm64(value) => {
                        assert!(value <= u32::MAX as u64);
                        let value = value as u32;
                        (Rex::WIDE, value.to_le_bytes())
                    }
                };

                let (mem, rm, rest) = self.encode_addr(addr);

                self.code.extend(u8::try_from(wide | mem));
                self.code.extend([0xc7, rm]);
                self.code.extend(rest);
                self.code.extend(imm);
            }

            (Operand::Register(reg), Operand::Location(name)) => {
                let (base, reg) = self.encode_reg(reg);
                let rex = Rex::WIDE | Rex::from_base(base);

                self.code.extend(u8::try_from(rex));

                let at = self.code.len() + 1;

                self.code.extend([0xb8 | reg, 0, 0, 0, 0, 0, 0, 0, 0]);

                self.relocations.entry(name).or_default().push(Relocation {
                    kind: RelocationKind::Absolute,
                    at,
                });
            }

            _ => unimplemented!(),
        }
    }

    fn inst_push(&mut self, operand: Operand) {
        match operand {
            Operand::Register(reg) => {
                let (base, reg) = self.encode_reg(reg);
                self.code.extend(u8::try_from(Rex::from_base(base)));
                self.code.push(0x50 | reg);
            }

            Operand::Memory(addr) => {
                let (mem, rm, rest) = self.encode_addr(addr);
                self.code.extend(u8::try_from(mem));
                self.code.extend([0xff, 0x30 | rm]);
                self.code.extend(rest);
            }

            _ => unimplemented!(),
        }
    }

    fn inst_pop(&mut self, operand: Operand) {
        match operand {
            Operand::Register(reg) => {
                let (base, reg) = self.encode_reg(reg);
                self.code.extend(u8::try_from(Rex::from_base(base)));
                self.code.push(0x58 | reg);
            }

            Operand::Memory(addr) => {
                let (mem, rm, rest) = self.encode_addr(addr);
                self.code.extend(u8::try_from(mem));
                self.code.extend([0x8f, rm]);
                self.code.extend(rest);
            }

            _ => panic!("invalid pop operand"),
        }
    }

    fn inst_ret(&mut self) {
        self.code.push(0xc3);
    }

    fn inst_sub(&mut self, dest: Operand, src: Operand) {
        match (dest, src) {
            (Operand::Register(dest), Operand::Register(src)) => {
                self.two_regs(0x29, dest, src);
            }

            (Operand::Register(dest), Operand::Memory(src)) => {
                self.reg_mem(0x2b, dest, src);
            }

            (Operand::Memory(dest), Operand::Register(src)) => {
                self.reg_mem(0x29, src, dest);
            }

            (Operand::Register(src), Operand::Immediate(imm)) => {
                let (wide, imm) = match imm {
                    Immediate::Imm8(value) => (Rex::NONE, (value as u32).to_le_bytes()),
                    Immediate::Imm16(value) => (Rex::NONE, (value as u32).to_le_bytes()),
                    Immediate::Imm32(value) => (Rex::NONE, value.to_le_bytes()),
                    Immediate::Imm64(value) => {
                        assert!(value <= u32::MAX as u64);
                        (Rex::WIDE, (value as u32).to_le_bytes())
                    }
                };

                let (base, reg) = self.encode_reg(src);
                let rex = wide | Rex::from_base(base);

                self.code.extend(u8::try_from(rex));
                self.code.extend([0x81, 0xe8 | reg]);
                self.code.extend(imm);
            }

            (Operand::Memory(addr), Operand::Immediate(imm)) => {
                let (wide, imm) = match imm {
                    Immediate::Imm8(value) => (Rex::NONE, (value as u32).to_le_bytes()),
                    Immediate::Imm16(value) => (Rex::NONE, (value as u32).to_le_bytes()),
                    Immediate::Imm32(value) => (Rex::NONE, value.to_le_bytes()),
                    Immediate::Imm64(value) => {
                        assert!(value <= u32::MAX as u64);
                        (Rex::WIDE, (value as u32).to_le_bytes())
                    }
                };

                let (mem, rm, rest) = self.encode_addr(addr);
                let rex = wide | mem;

                self.code.extend(u8::try_from(rex));
                self.code.extend([0x81, 0x28 | rm]);
                self.code.extend(rest);
                self.code.extend(imm);
            }

            _ => unimplemented!(),
        }
    }

    fn inst_syscall(&mut self) {
        self.code.extend([0x0f, 0x05]);
    }
}
