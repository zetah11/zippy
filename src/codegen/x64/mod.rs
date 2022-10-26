mod encode;
mod lower;
mod repr;

#[cfg(test)]
mod tests;

use crate::lir;
use crate::resolve::names::Names;

pub fn codegen(names: &mut Names, program: lir::Program) -> Vec<u8> {
    let program = lower::lower(names, program);
    encode::encode(program)
}

use crate::asm::{Constraints, RegisterInfo};

pub const CONSTRAINTS: Constraints = Constraints {
    #[rustfmt::skip]
    registers: &[
        RegisterInfo { size: 8, name: "rdi", aliases: &[] },
        RegisterInfo { size: 8, name: "rsi", aliases: &[] },
        RegisterInfo { size: 8, name: "rdx", aliases: &[] },
        RegisterInfo { size: 8, name: "rcx", aliases: &[] },
        RegisterInfo { size: 8, name: "r8", aliases: &[] },
        RegisterInfo { size: 8, name: "r9", aliases: &[] },
        RegisterInfo { size: 8, name: "r10", aliases: &[] },
        RegisterInfo { size: 8, name: "r11", aliases: &[] },
        RegisterInfo { size: 8, name: "r12", aliases: &[] },
        RegisterInfo { size: 8, name: "r13", aliases: &[] },
        RegisterInfo { size: 8, name: "r14", aliases: &[] },
        RegisterInfo { size: 8, name: "r15", aliases: &[] },
    ],
    call_clobbers: &[0, 1, 2, 3, 4, 5],
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
