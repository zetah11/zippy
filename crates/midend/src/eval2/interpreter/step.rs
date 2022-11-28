use std::collections::hash_map::Entry;

use common::mir::{self, pretty::Prettier, BranchNode, StmtNode};
use log::trace;

use super::{Env, Error, Frame, InstructionPlace, Interpreter, Place, Value, StateAction};

impl Interpreter<'_> {
    pub(super) fn single_step(&mut self) -> Result<(), Error> {
        let place = self.place().ok_or(Error::NothingLeft)?;

        match place {
            Place::Branch(_) => {
                self.perform_branch(place)?;
            }

            Place::Instruction(_, index, at) => {
                self.perform_instruction(place, index, at)?;
            }
        }

        Ok(())
    }

    fn perform_branch(&mut self, place: Place) -> Result<(), Error> {
        let block = self.block_of_or_top_level(&place);

        match &block.branch.node {
            BranchNode::Return(values) => {
                trace!("evaluating return");

                let return_values: Vec<_> = values
                    .clone()
                    .into_iter()
                    .map(|value| self.make_value(&value))
                    .collect();

                self.return_values.clear();

                for value in return_values {
                    match value {
                        Some(value) => self.return_values.push(value),
                        None => return Ok(()),
                    }
                }

                let _ = self.current_mut().unwrap().exit();
                Ok(())
            }

            BranchNode::Jump(..) => {
                trace!("evaluating jump");
                todo!()
            }
        }
    }

    fn perform_instruction(
        &mut self,
        place: Place,
        index: usize,
        at: InstructionPlace,
    ) -> Result<(), Error> {
        let block = self.block_of_or_top_level(&place);
        let stmt = block.exprs.get(index).unwrap().clone();

        match (stmt.node, at) {
            (StmtNode::Apply { fun, args, .. }, InstructionPlace::Execute) => {
                trace!("evaluating apply {}(...)", {
                    let prettier = Prettier::new(self.names, self.types);
                    prettier.pretty_name(&fun)
                });

                let args: Vec<_> = args
                    .into_iter()
                    .map(|value| self.make_value(&value))
                    .collect();

                let mut frame = Frame {
                    place: Place::Instruction(fun, 0, InstructionPlace::Bind),
                    env: Env::new(),
                };

                let Some(fun) = self.make_value(&mir::Value {
                    ty: stmt.ty, // todo: type
                    span: stmt.span,
                    node: mir::ValueNode::Name(fun),
                }) else {
                    return Ok(())
                };

                let params = match fun {
                    Value::Function(fun) => self.functions.get(&fun).unwrap(),
                    _ => unreachable!(),
                };

                for (arg, param) in args.into_iter().zip(params.iter()) {
                    match arg {
                        Some(arg) => {
                            frame.env.add(*param, arg);
                        }

                        None => return Ok(()),
                    }
                }

                self.next_place();
                self.current_mut().unwrap().enter(frame);
            }

            (StmtNode::Apply { names, .. }, InstructionPlace::Bind) => {
                trace!("binding apply");
                assert_eq!(names.len(), self.return_values.len());

                let return_values: Vec<_> = self.return_values.drain(..).collect();
                for (name, value) in names.into_iter().zip(return_values) {
                    self.current_mut().unwrap().add(name, value);
                }

                self.next_place();
            }

            (StmtNode::Proj { .. }, _) => todo!(),
            (StmtNode::Tuple { .. }, _) => todo!(),

            (StmtNode::Function { name, params, body }, InstructionPlace::Execute) => {
                trace!("evaluating function {}", {
                    let prettier = Prettier::new(self.names, self.types);
                    prettier.pretty_name(&name)
                });

                let next = if let Entry::Vacant(e) = self.functions.entry(name) {
                    assert!(self.blocks.insert(name, body).is_none());
                    e.insert(params.clone());

                    // Partially evaluate the function
                    let place = self.place_of(&name);

                    let mut frame = Frame {
                        env: Env::new(),
                        place,
                    };

                    for param in params {
                        frame.env.add(param, Value::Quoted(param));
                    }

                    let mut state = self.current().unwrap().split(StateAction::StoreGlobal(name));
                    state.enter(frame);

                    vec![state]
                } else {
                    vec![]
                };

                self.next_place();
                self.worklist.extend(next);
            }

            (StmtNode::Function { name, .. }, InstructionPlace::Bind) => {
                trace!("binding function");
                self.current_mut().unwrap().add(name, Value::Function(name));
                self.next_place();
            }

            (StmtNode::Join { .. }, _) => todo!(),
        }

        Ok(())
    }

    /// Move to the next instruction or branch.
    /// Panics if the current place does not correspond to any known block, if there is no current place, or if the
    /// current place is a branch.
    fn next_place(&mut self) {
        let current = self.place().unwrap();
        let block = self.block_of(&current).unwrap();

        let next = match current {
            Place::Branch(..) => unreachable!(),
            Place::Instruction(name, index, InstructionPlace::Execute) => {
                Place::Instruction(name, index, InstructionPlace::Bind)
            }
            Place::Instruction(name, index, InstructionPlace::Bind)
                if index < block.exprs.len() - 1 =>
            {
                Place::Instruction(name, index + 1, InstructionPlace::Execute)
            }
            Place::Instruction(name, _, InstructionPlace::Bind) => Place::Branch(name),
        };

        // beautiful
        self.current_mut().unwrap().current_mut().unwrap().place = next;
    }
}
