use iced_x86::Register;

use crate::asm::{Constraints, RegisterInfo};

pub const CONSTRAINTS: Constraints = Constraints {
    call_clobbers: &[],

    parameters: &[0, 1, 2, 3, 4, 5],
    returns: &[0, 1, 2, 3, 4, 5],

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
};

pub fn regid_to_reg(id: usize) -> Register {
    match id {
        0 => Register::RDI,
        1 => Register::RSI,
        2 => Register::RDX,
        3 => Register::RCX,
        4 => Register::R8,
        5 => Register::R9,
        6 => Register::R10,
        7 => Register::R11,
        8 => Register::R12,
        9 => Register::R13,
        10 => Register::R14,
        11 => Register::R15,

        _ => unreachable!(),
    }
}
