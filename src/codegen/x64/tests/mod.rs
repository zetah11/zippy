use std::collections::HashMap;

use super::encode::encode;
use super::repr::{Block, Immediate, Instruction, Name, Operand, Procedure, Program, Register};

#[test]
fn encode_call_ret() {
    let main = Name(0);
    let next = Name(1);
    let (b1, b2) = (Name(2), Name(3));

    let main_proc = Procedure {
        prelude: vec![],
        block_order: vec![b1],
        blocks: HashMap::from([(
            b1,
            Block {
                insts: vec![Instruction::Call(Operand::Location(next))],
            },
        )]),
    };

    let next_proc = Procedure {
        prelude: vec![],
        block_order: vec![b2],
        blocks: HashMap::from([(
            b2,
            Block {
                insts: vec![Instruction::Ret],
            },
        )]),
    };

    let program = Program {
        procedures: vec![(main, main_proc), (next, next_proc)],
        names: Default::default(),
    };

    #[rustfmt::skip]
    let expected = vec![
        0xe8, 0, 0, 0, 0,
        0xc3,
    ];

    let actual = encode(program).code;

    assert_eq!(expected, actual);
}

#[test]
fn encode_prologue_epilogue() {
    let main = Name(0);
    let b1 = Name(1);

    let main_proc = Procedure {
        prelude: vec![],
        block_order: vec![b1],
        blocks: HashMap::from([(
            b1,
            Block {
                insts: vec![
                    Instruction::Push(Operand::Register(Register::Rbp)),
                    Instruction::Mov(
                        Operand::Register(Register::Rbp),
                        Operand::Register(Register::Rsp),
                    ),
                    Instruction::Sub(
                        Operand::Register(Register::Rsp),
                        Operand::Immediate(Immediate::Imm64(24)),
                    ),
                    Instruction::Leave,
                    Instruction::Ret,
                ],
            },
        )]),
    };

    let program = Program {
        procedures: vec![(main, main_proc)],
        names: Default::default(),
    };

    #[rustfmt::skip]
    let expected = vec![
        0x55,
        0x48, 0x89, 0xe5,
        0x48, 0x81, 0xec, 0x18, 0, 0, 0,
        0xc9,
        0xc3,
    ];

    let actual = encode(program).code;

    assert_eq!(expected, actual);
}

#[test]
fn encode_two_way_jump() {
    let main = Name(0);
    let (b1, b2) = (Name(1), Name(2));

    let main_proc = Procedure {
        prelude: vec![],
        block_order: vec![b1, b2],
        blocks: HashMap::from([
            (
                b1,
                Block {
                    insts: vec![Instruction::Jump(Operand::Location(b2))],
                },
            ),
            (
                b2,
                Block {
                    insts: vec![
                        Instruction::Pop(Operand::Register(Register::Rbx)),
                        Instruction::Jump(Operand::Location(b1)),
                    ],
                },
            ),
        ]),
    };

    let program = Program {
        procedures: vec![(main, main_proc)],
        names: Default::default(),
    };

    #[rustfmt::skip]
    let expected = vec![
        0xe9, 0x05, 0, 0, 0,
        0x5b,
        0xe9, 0xfa, 0xff, 0xff, 0xff,
    ];

    let actual = encode(program).code;

    assert_eq!(expected, actual);
}
