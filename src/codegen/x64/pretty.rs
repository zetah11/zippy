use super::repr::{
    Address, Block, Immediate, Instruction, Name, Operand, Procedure, Program, Register, Scale,
};
use crate::resolve::names as resolve;

#[must_use = "the pretty printer does not output anything by itself"]
pub fn pretty_program(names: &resolve::Names, program: &Program) -> String {
    let prettier = Prettier { names, program };
    prettier.pretty()
}

#[derive(Debug)]
struct Prettier<'a> {
    names: &'a resolve::Names,
    program: &'a Program,
}

impl Prettier<'_> {
    pub fn pretty(self) -> String {
        let mut result = Result::new();
        for (name, procedure) in self.program.procedures.iter() {
            self.pretty_procedure(&mut result, name, procedure);
            result.push('\n');
        }
        result.into()
    }

    fn pretty_procedure(&self, within: &mut Result, name: &Name, procedure: &Procedure) {
        self.pretty_name(within, "f", name);
        within.push_str(":");

        if procedure.prelude.is_empty() {
            within.push('\n');
        }

        for instruction in procedure.prelude.iter() {
            self.pretty_instruction(within, instruction);
        }

        for name in procedure.block_order.iter() {
            let block = procedure.blocks.get(name).unwrap();
            self.pretty_block(within, name, block);
        }
    }

    fn pretty_block(&self, within: &mut Result, name: &Name, block: &Block) {
        self.pretty_name(within, ".b", name);
        within.push(':');

        for instruction in block.insts.iter() {
            self.pretty_instruction(within, instruction);
        }
    }

    fn pretty_instruction(&self, within: &mut Result, instruction: &Instruction) {
        let (name, operands) = match instruction {
            Instruction::Call(a) => ("call", vec![a]),
            Instruction::Cmp(a, b) => ("cmp", vec![a, b]),
            Instruction::Jump(a) => ("jmp", vec![a]),
            Instruction::Leave => ("leave", vec![]),
            Instruction::Mov(a, b) => ("mov", vec![a, b]),
            Instruction::Pop(a) => ("pop", vec![a]),
            Instruction::Push(a) => ("push", vec![a]),
            Instruction::Ret => ("ret", vec![]),
            Instruction::Sub(a, b) => ("sub", vec![a, b]),
        };

        within.push_at(8, name);

        if !operands.is_empty() {
            within.push(' ');
            let mut first = true;
            for operand in operands {
                if !first {
                    within.push_str(", ");
                }
                first = false;

                self.pretty_operand(within, operand);
            }
        }

        within.push('\n');
    }

    fn pretty_operand(&self, within: &mut Result, operand: &Operand) {
        let string = match operand {
            Operand::Immediate(Immediate::Imm8(value)) => format!("{value}"),
            Operand::Immediate(Immediate::Imm16(value)) => format!("{value}"),
            Operand::Immediate(Immediate::Imm32(value)) => format!("{value}"),
            Operand::Immediate(Immediate::Imm64(value)) => format!("{value}"),

            Operand::Location(name) => {
                // TODO: correct prefix
                self.pretty_name(within, ".b", name);
                return;
            }

            Operand::Memory(address) => {
                self.pretty_address(within, address);
                return;
            }

            Operand::Register(register) => {
                self.pretty_register(within, register);
                return;
            }
        };

        within.push_str(&string);
    }

    fn pretty_address(&self, within: &mut Result, address: &Address) {
        within.push('[');

        let mut plus = false;
        if let Some(ref register) = address.reg {
            self.pretty_register(within, register);
            plus = true;
        }

        if let Some(ref register) = address.offset {
            if plus {
                within.push_str(" + ");
            }

            self.pretty_register(within, register);

            match address.scale {
                Scale::One => {}
                Scale::Two => within.push_str(" * 2"),
                Scale::Four => within.push_str(" * 4"),
                Scale::Eight => within.push_str(" * 8"),
            }

            plus = true;
        }

        if let Some(ref displacement) = address.displacement {
            if plus {
                within.push_str(" + ");
            }

            within.push_str(format!("{displacement}"));
        }

        within.push(']');
    }

    fn pretty_register(&self, within: &mut Result, register: &Register) {
        let name = match register {
            Register::Rax => "rax",
            Register::Rbx => "rbx",
            Register::Rcx => "rcx",
            Register::Rdx => "rdx",
            Register::Rdi => "rdi",
            Register::Rsi => "rsi",
            Register::Rsp => "rsp",
            Register::Rbp => "rbp",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
        };

        within.push_str(name);
    }

    fn pretty_name(&self, within: &mut Result, prefix: impl AsRef<str>, name: &Name) {
        let name = self.program.names.get(name);
        let resolve::Path(_ctx, name) = self.names.get_path(name);
        let name = match name {
            resolve::Actual::Generated(id) => id.to_string(prefix),
            resolve::Actual::Lit(name) => name.clone(),
            resolve::Actual::Scope(_) => unimplemented!(),
        };

        within.push_str(name);
    }
}

#[derive(Debug, Default)]
struct Result {
    data: String,
    offset: usize,
}

impl Result {
    pub fn new() -> Self {
        Self {
            data: String::new(),
            offset: 0,
        }
    }

    pub fn push(&mut self, ch: char) {
        if ch == '\n' {
            self.offset = 0;
        } else {
            self.offset += 1;
        }

        self.data.push(ch);
    }

    pub fn push_str(&mut self, string: impl AsRef<str>) {
        let string = string.as_ref();
        let split: Vec<_> = string.split('\n').collect();
        if split.len() > 1 {
            self.offset = split.last().unwrap().len()
        } else {
            self.offset += string.len();
        }

        self.data.push_str(string);
    }

    /// Add `string` padded with at least one space such that it starts at `offset` on the line. If it cannot fit, a
    /// newline is inserted.
    pub fn push_at(&mut self, offset: usize, string: impl AsRef<str>) {
        if self.offset >= offset {
            self.push('\n');
        } else {
            let spaces = offset - self.offset;
            for _ in 0..spaces {
                self.push(' ');
            }
            self.push_str(string);
        }
    }
}

impl From<Result> for String {
    fn from(result: Result) -> Self {
        result.data
    }
}
