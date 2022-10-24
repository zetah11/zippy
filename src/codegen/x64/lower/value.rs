use super::{lir, x64, Lowerer};
use crate::codegen::x64::regid_to_reg;

impl Lowerer<'_> {
    pub fn lower_target(&mut self, target: lir::Target) -> x64::Operand {
        match target {
            lir::Target::Name(name) => {
                let name = self.lower_name(name);
                x64::Operand::Location(name)
            }

            lir::Target::Register(lir::Register::Physical(reg)) => {
                x64::Operand::Register(regid_to_reg(reg))
            }

            lir::Target::Register(lir::Register::Frame(offset, ty)) => {
                let size = self.program.types.sizeof(&ty);
                if size != 8 {
                    todo!()
                }

                x64::Operand::Memory(x64::Address {
                    reg: Some(x64::Register::Rbp),
                    offset: None,
                    scale: x64::Scale::One,
                    displacement: Some(i32::try_from(offset).unwrap()),
                })
            }

            lir::Target::Register(lir::Register::Virtual(..)) => unreachable!(),
        }
    }

    pub fn lower_value(&mut self, value: lir::Value) -> x64::Operand {
        match value {
            // `i64 as u64` is a bitwise reinterpretation so this is good
            // Why isn't this in the stdlib?
            lir::Value::Integer(i) => x64::Operand::Immediate(x64::Immediate::Imm64(i as u64)),

            lir::Value::Name(name) => {
                let name = self.lower_name(name);
                x64::Operand::Location(name)
            }

            lir::Value::Register(lir::Register::Physical(reg)) => {
                x64::Operand::Register(regid_to_reg(reg))
            }

            lir::Value::Register(lir::Register::Frame(offset, ty)) => {
                let size = self.program.types.sizeof(&ty);
                if size != 8 {
                    todo!()
                }

                x64::Operand::Memory(x64::Address {
                    reg: Some(x64::Register::Rbp),
                    offset: None,
                    scale: x64::Scale::One,
                    displacement: Some(i32::try_from(offset).unwrap()),
                })
            }

            lir::Value::Register(lir::Register::Virtual(..)) => unreachable!(),
        }
    }
}
