use common::lir::{self, BaseOffset, BlockId, Target, TargetNode, Value, ValueNode};
use common::names::Name;
use iced_x86::code_asm::{
    gpr16, gpr32, gpr64, gpr8, ptr, rax, rbp, AsmMemoryOperand, AsmRegister16, AsmRegister32,
    AsmRegister64, AsmRegister8,
};
use iced_x86::Register;

use super::{Constraints, Lowerer};
use crate::asm::AllocConstraints;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operand {
    Gpr64(AsmRegister64),
    Gpr32(AsmRegister32),
    Gpr16(AsmRegister16),
    Gpr8(AsmRegister8),

    Memory(AsmMemoryOperand),
    Label(Name),
    Block(BlockId),
    Integer(i64),
}

impl TryFrom<Register> for Operand {
    type Error = ();

    fn try_from(register: Register) -> Result<Self, Self::Error> {
        if let Some(reg) = gpr64::get_gpr64(register) {
            return Ok(Self::Gpr64(reg));
        }

        if let Some(reg) = gpr32::get_gpr32(register) {
            return Ok(Self::Gpr32(reg));
        }

        if let Some(reg) = gpr16::get_gpr16(register) {
            return Ok(Self::Gpr16(reg));
        }

        if let Some(reg) = gpr8::get_gpr8(register) {
            return Ok(Self::Gpr8(reg));
        }

        Err(())
    }
}

impl Lowerer<'_> {
    pub fn target_operand(&self, target: &Target) -> Option<Operand> {
        match target.node {
            TargetNode::Register(lir::Register::Physical(_)) => {
                todo!()
            }
            TargetNode::Register(lir::Register::Frame(BaseOffset::Local(offset), ty)) => {
                let size = Constraints::sizeof(&self.program.types, &ty);
                let physical = offset + size;

                Some(Operand::Memory(rbp - physical))
            }

            TargetNode::Register(lir::Register::Frame(BaseOffset::Argument(..), _)) => {
                todo!("need to remember continuation count");
            }

            TargetNode::Register(lir::Register::Virtual(_)) => unreachable!(),

            TargetNode::Name(name) => Some(Operand::Label(name)),
        }
    }

    pub fn value_operand(&self, value: &Value) -> Option<Operand> {
        match value.node {
            ValueNode::Register(lir::Register::Physical(_)) => {
                todo!()
            }
            ValueNode::Register(lir::Register::Frame(BaseOffset::Local(offset), ty)) => {
                let size = Constraints::sizeof(&self.program.types, &ty);
                let physical = offset + size;
                Some(Operand::Memory(rbp - physical))
            }
            ValueNode::Register(lir::Register::Frame(BaseOffset::Argument(_), _)) => {
                todo!("need to remember continuation count");
            }
            ValueNode::Register(lir::Register::Virtual(_)) => unreachable!(),

            ValueNode::Name(name) => Some(Operand::Label(name)),

            ValueNode::Integer(i) => Some(Operand::Integer(i)),
        }
    }

    pub fn asm_call(&mut self, value: Operand) {
        match value {
            Operand::Gpr16(source) => self.asm.call(source).unwrap(),
            Operand::Gpr32(source) => self.asm.call(source).unwrap(),
            Operand::Gpr64(source) => self.asm.call(source).unwrap(),
            Operand::Memory(source) => self.asm.call(source).unwrap(),
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.call(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.call(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for call"),
        }
    }

    pub fn asm_cmp(&mut self, left: Operand, right: Operand) {
        match (left, right) {
            (Operand::Gpr8(left), Operand::Gpr8(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr8(left), Operand::Memory(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr8(left), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(right), _) => self.asm.cmp(left, right).unwrap(),
                    (_, Ok(right)) => self.asm.cmp(left, right).unwrap(),
                    _ => unreachable!("integer literal too big to compare with 8-bit register"),
                }
            }

            (Operand::Gpr16(left), Operand::Gpr16(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr16(left), Operand::Memory(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr16(left), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(right), _) => self.asm.cmp(left, right).unwrap(),
                    (_, Ok(right)) => self.asm.cmp(left, right).unwrap(),
                    _ => unreachable!("integer literal too big to compare with 8-bit register"),
                }
            }

            (Operand::Gpr32(left), Operand::Gpr32(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr32(left), Operand::Memory(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr32(left), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(right), _) => self.asm.cmp(left, right).unwrap(),
                    (_, Ok(right)) => self.asm.cmp(left, right).unwrap(),
                    _ => unreachable!("integer literal too big to compare with 8-bit register"),
                }
            }

            (Operand::Gpr64(left), Operand::Gpr64(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr64(left), Operand::Memory(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr64(left), Operand::Integer(i)) => match i32::try_from(i) {
                Ok(i) => self.asm.cmp(left, i).unwrap(),
                _ => {
                    self.asm.mov(rax, i).unwrap();
                    self.asm.cmp(left, rax).unwrap();
                }
            },

            (Operand::Memory(left), Operand::Gpr8(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Memory(left), Operand::Gpr16(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Memory(left), Operand::Gpr32(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Memory(left), Operand::Gpr64(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Memory(left), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(i), _) => self.asm.cmp(left, i).unwrap(),
                    (_, Ok(i)) => self.asm.cmp(left, i).unwrap(),
                    (..) => {
                        self.asm.mov(rax, i).unwrap();
                        self.asm.cmp(left, rax).unwrap();
                    }
                }
            }

            _ => unreachable!("invalid opcode/operand combination for cmp"),
        }
    }

    pub fn asm_je(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.je(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.je(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jg(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jg(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jg(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jge(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jge(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jge(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jl(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jl(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jl(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jle(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jle(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jle(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jmp(&mut self, value: Operand) {
        match value {
            Operand::Gpr16(source) => self.asm.jmp(source).unwrap(),
            Operand::Gpr32(source) => self.asm.jmp(source).unwrap(),
            Operand::Gpr64(source) => self.asm.jmp(source).unwrap(),
            Operand::Memory(source) => self.asm.jmp(source).unwrap(),
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jmp(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jmp(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for jmp"),
        }
    }

    pub fn asm_jne(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jne(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jne(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_leave(&mut self) {
        self.asm.leave().unwrap();
    }

    pub fn asm_mov(&mut self, target: Operand, source: Operand) {
        match (target, source) {
            (Operand::Gpr8(target), Operand::Gpr8(source)) => self.asm.mov(target, source).unwrap(),
            (Operand::Gpr8(target), Operand::Memory(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr8(target), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(source), _) => self.asm.mov(target, source).unwrap(),
                    (_, Ok(source)) => self.asm.mov(target, source).unwrap(),
                    _ => unreachable!("integer literal too big for 8-bit mov"),
                }
            }
            (Operand::Gpr8(target), Operand::Label(source)) => {
                let source = self.label(source);
                self.asm.mov(target, ptr(source)).unwrap();
            }

            (Operand::Gpr16(target), Operand::Gpr16(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr16(target), Operand::Memory(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr16(target), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(source), _) => self.asm.mov(target, source).unwrap(),
                    (_, Ok(source)) => self.asm.mov(target, source).unwrap(),
                    _ => unreachable!("integer literal too big for 8-bit mov"),
                }
            }
            (Operand::Gpr16(target), Operand::Label(source)) => {
                let source = self.label(source);
                self.asm.mov(target, ptr(source)).unwrap();
            }

            (Operand::Gpr32(target), Operand::Gpr32(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr32(target), Operand::Memory(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr32(target), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(source), _) => self.asm.mov(target, source).unwrap(),
                    (_, Ok(source)) => self.asm.mov(target, source).unwrap(),
                    _ => unreachable!("integer literal too big for 8-bit mov"),
                }
            }
            (Operand::Gpr32(target), Operand::Label(source)) => {
                let source = self.label(source);
                self.asm.mov(target, ptr(source)).unwrap();
            }

            (Operand::Gpr64(target), Operand::Gpr64(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr64(target), Operand::Memory(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr64(target), Operand::Integer(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr64(target), Operand::Label(source)) => {
                let ty = self.program.context.get(&source);
                let source = self.label(source);

                if self.program.types.is_function(&ty) {
                    self.asm.lea(target, ptr(source)).unwrap();
                } else {
                    self.asm.mov(target, ptr(source)).unwrap();
                }
            }
            (Operand::Gpr64(target), Operand::Block(source)) => {
                let source = self.block_label(source);
                self.asm.lea(target, ptr(source)).unwrap();
            }

            (Operand::Memory(target), Operand::Gpr64(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Memory(target), Operand::Integer(source)) => {
                match (i32::try_from(source), u32::try_from(source)) {
                    (Ok(i), _) => self.asm.mov(target, i).unwrap(),
                    (_, Ok(i)) => self.asm.mov(target, i).unwrap(),
                    (..) => {
                        self.asm.mov(rax, source).unwrap();
                        self.asm.mov(target, rax).unwrap();
                    }
                }
            }

            _ => unreachable!("invalid opcode/operand combination for mov"),
        }
    }

    pub fn asm_pop(&mut self, target: Operand) {
        match target {
            Operand::Gpr16(target) => self.asm.pop(target).unwrap(),
            Operand::Gpr32(target) => self.asm.pop(target).unwrap(),
            Operand::Gpr64(target) => self.asm.pop(target).unwrap(),
            Operand::Memory(target) => self.asm.pop(target).unwrap(),

            _ => unreachable!("invalid opcode/operand combination for pop"),
        }
    }

    pub fn asm_push(&mut self, value: Operand) {
        match value {
            Operand::Gpr16(source) => self.asm.push(source).unwrap(),
            Operand::Gpr32(source) => self.asm.push(source).unwrap(),
            Operand::Gpr64(source) => self.asm.push(source).unwrap(),
            Operand::Memory(source) => self.asm.push(source).unwrap(),
            Operand::Integer(i) => match (i32::try_from(i), u32::try_from(i)) {
                (Ok(i), _) => self.asm.push(i).unwrap(),
                (_, Ok(i)) => self.asm.push(i).unwrap(),
                (..) => {
                    self.asm.mov(rax, i).unwrap();
                    self.asm.push(rax).unwrap();
                }
            },
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.lea(rax, ptr(source)).unwrap();
                self.asm.push(rax).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.lea(rax, ptr(source)).unwrap();
                self.asm.push(rax).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for push"),
        }
    }

    pub fn asm_ret(&mut self) {
        self.asm.ret().unwrap();
    }

    pub fn asm_ret1(&mut self, by: u32) {
        self.asm.ret_1(by).unwrap();
    }
}
