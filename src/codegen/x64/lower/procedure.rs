use std::collections::HashMap;

use super::{lir, x64, Lowerer};

impl Lowerer<'_> {
    pub fn lower_procedure(&mut self, name: x64::Name, mut proc: lir::Procedure) -> x64::Procedure {
        let order = self.block_order(&proc);
        let mut block_order = Vec::with_capacity(order.len());
        let mut blocks = HashMap::with_capacity(order.len());

        for id in order.iter() {
            self.lower_block_id(name, *id);
        }

        for id in order.iter() {
            let new_id = self.blocks.get(id).copied().unwrap();
            block_order.push(new_id);

            let block = proc.blocks.remove(id).unwrap();
            let block = self.lower_block(&order, *id, &proc, block);
            blocks.insert(new_id, block);
        }

        x64::Procedure {
            block_order,
            blocks,
        }
    }

    fn block_order(&self, proc: &lir::Procedure) -> Vec<lir::BlockId> {
        let mut order = vec![proc.entry];
        let mut worklist = vec![proc.entry];

        while let Some(block) = worklist.pop() {
            match proc.get_branch(proc.get(&block).branch) {
                lir::Branch::Return(..) => {}
                lir::Branch::Call(_fun, _arg, conts) => {
                    for id in conts.iter() {
                        if proc.has_block(id) && !order.contains(id) {
                            order.push(*id);
                            worklist.push(*id);
                        }
                    }
                }

                lir::Branch::Jump(to, _) => {
                    if !order.contains(to) {
                        order.push(*to);
                        worklist.push(*to);
                    }
                }

                lir::Branch::JumpIf {
                    then: (then, _),
                    elze: (elze, _),
                    ..
                } => {
                    if !order.contains(then) {
                        order.push(*then);
                        worklist.push(*then);
                    }

                    if !order.contains(elze) {
                        order.push(*elze);
                        worklist.push(*elze);
                    }
                }
            }
        }

        order
    }
}
