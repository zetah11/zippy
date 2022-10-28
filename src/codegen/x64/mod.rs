mod encode;
mod lower;
mod pretty;
mod repr;

#[cfg(test)]
mod tests;

pub use self::pretty::pretty_program;
pub use encode::encode;

use crate::lir;
use crate::resolve::names::{Name, Names};

pub fn codegen(names: &mut Names, entry: Option<Name>, program: lir::Program) -> repr::Program {
    lower::lower(names, entry, program)
}

use crate::asm::{Constraints, RegisterInfo};

pub const CONSTRAINTS: Constraints = Constraints {
    #[rustfmt::skip]
    registers: &[
        RegisterInfo { id:  0, size: 8, name: "rdi", aliases: &[] },
        RegisterInfo { id:  1, size: 8, name: "rsi", aliases: &[] },
        RegisterInfo { id:  2, size: 8, name: "rdx", aliases: &[] },
        RegisterInfo { id:  3, size: 8, name: "rcx", aliases: &[] },
        RegisterInfo { id:  4, size: 8, name: "r8",  aliases: &[] },
        RegisterInfo { id:  5, size: 8, name: "r9",  aliases: &[] },
        RegisterInfo { id:  6, size: 8, name: "r10", aliases: &[] },
        RegisterInfo { id:  7, size: 8, name: "r11", aliases: &[] },
        RegisterInfo { id:  8, size: 8, name: "r12", aliases: &[] },
        RegisterInfo { id:  9, size: 8, name: "r13", aliases: &[] },
        RegisterInfo { id: 10, size: 8, name: "r14", aliases: &[] },
        RegisterInfo { id: 11, size: 8, name: "r15", aliases: &[] },
    ],
    call_clobbers: &[0, 1, 2, 3, 4, 5],
    parameters: &[0, 1, 2, 3, 4, 5],
    returns: &[0, 1, 2, 3, 4, 5],
};

fn regid_to_reg(id: usize) -> repr::Register {
    match id {
        0 => repr::Register::Rdi,
        1 => repr::Register::Rsi,
        2 => repr::Register::Rdx,
        3 => repr::Register::Rcx,
        4 => repr::Register::R8,
        5 => repr::Register::R9,
        6 => repr::Register::R10,
        7 => repr::Register::R11,
        8 => repr::Register::R12,
        9 => repr::Register::R13,
        10 => repr::Register::R14,
        11 => repr::Register::R15,
        _ => unreachable!(),
    }
}
