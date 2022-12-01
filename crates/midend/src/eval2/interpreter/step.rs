use std::collections::hash_map::Entry;

use common::mir::{self, pretty::Prettier, BranchNode, StmtNode};
use log::trace;

use super::{Error, Frame, InstructionPlace, Interpreter, Place, StateAction, Value};

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

                let branch = mir::Branch {
                    node: BranchNode::Return(values.clone()),
                    ..block.branch
                };

                let span = block.span;
                let ty = block.ty;

                let return_values: Vec<_> = values
                    .clone()
                    .into_iter()
                    .map(|value| self.make_value(&value))
                    .collect();

                self.return_values.clear();

                for value in return_values {
                    match value {
                        Some(value) => self.return_values.push(value),
                        None => {
                            let stmts = self.stmts();
                            let block = mir::Block {
                                stmts,
                                branch,
                                span,
                                ty,
                            };

                            let name = self.current().unwrap().current().unwrap().name;

                            assert!(self.blocks.insert(name, block).is_some());

                            self.return_values.clear();
                            let _ = self.current_mut().unwrap().exit();
                            return Ok(());
                        }
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
        let stmt = block.stmts.get(index).unwrap().clone();

        match (stmt.node, at) {
            (StmtNode::Apply { fun, args, names }, InstructionPlace::Execute) => {
                trace!("evaluating apply {}(...)", {
                    let prettier = Prettier::new(self.names, self.types);
                    prettier.pretty_name(&fun)
                });

                let new_args: Vec<_> = args.iter().map(|value| self.make_value(value)).collect();

                let Some(fun) = self.make_value(&mir::Value {
                    ty: stmt.ty, // todo: type
                    span: stmt.span,
                    node: mir::ValueNode::Name(fun),
                }) else {
                    self.next_place();
                    self.next_place();

                    // mmmm
                    self.push_stmt(mir::Statement {
                        node: StmtNode::Apply {
                            names,
                            fun,
                            args,
                        },
                        span: stmt.span,
                        ty: stmt.ty,
                    });

                    return Ok(())
                };

                let (fun, params) = match fun {
                    Value::Function(fun) => (fun, self.functions.get(&fun).unwrap()),
                    _ => unreachable!(),
                };

                let mut frame = Frame::new(place);

                for (arg, param) in new_args.into_iter().zip(params.iter()) {
                    if let Some(arg) = arg {
                        frame.env.add(*param, arg);
                    } else {
                        self.push_stmt(mir::Statement {
                            node: StmtNode::Apply { names, fun, args },
                            span: stmt.span,
                            ty: stmt.ty,
                        });

                        self.next_place();
                        return Ok(());
                    }
                }

                self.next_place();
                self.current_mut().unwrap().enter(frame);
            }

            (StmtNode::Apply { names, fun, args }, InstructionPlace::Bind) => {
                trace!("binding apply");
                if names.len() == self.return_values.len() {
                    let return_values: Vec<_> = self.return_values.drain(..).collect();
                    for (name, value) in names.into_iter().zip(return_values) {
                        self.current_mut().unwrap().add(name, value);
                    }
                } else {
                    self.push_stmt(mir::Statement {
                        node: StmtNode::Apply { names, fun, args },
                        span: stmt.span,
                        ty: stmt.ty,
                    });
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
                    let place = self.place_of_top_level(&name).unwrap();
                    let mut frame = Frame::new(place);

                    for param in params {
                        frame.env.add(param, Value::Quoted(param));
                    }

                    let mut state = self
                        .current()
                        .unwrap()
                        .split(StateAction::StoreGlobal(name));
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
                if index < block.stmts.len() - 1 =>
            {
                Place::Instruction(name, index + 1, InstructionPlace::Execute)
            }
            Place::Instruction(name, _, InstructionPlace::Bind) => Place::Branch(name),
        };

        // beautiful
        self.current_mut().unwrap().current_mut().unwrap().place = next;
    }
}
