use std::cmp::Ordering;

use super::{lir, x64, Lowerer};

impl Lowerer<'_> {
    pub fn lower_constant(&mut self, name: x64::Name, value: lir::Value) -> Vec<u8> {
        let name = self.names.get(&name);
        let ty = self.program.context.get(name);
        let size = self.program.types.sizeof(&ty);

        match value {
            lir::Value::Integer(i) => {
                let data = i.to_le_bytes();

                match size.cmp(&data.len()) {
                    Ordering::Equal => data.to_vec(),
                    Ordering::Greater => {
                        let mut res = Vec::with_capacity(size);
                        let remaining = size - data.len();
                        res.extend(data);
                        res.extend(std::iter::repeat(0).take(remaining));
                        res
                    }
                    Ordering::Less => {
                        println!("{size:?} {data:?}");
                        todo!("truncation")
                    }
                }
            }

            lir::Value::Name(_) => todo!(),

            lir::Value::Register(_) => unreachable!(),
        }
    }
}
