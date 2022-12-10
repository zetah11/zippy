use std::ops::Add;

use log::trace;
use zippy_common::mir::Block;
use zippy_common::Driver;

use super::action::Action;
use super::place::Place;
use super::state::Frame;
use super::value::Operation;
use super::Interpreter;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Step {
    /// The interpreter has nothing more to do.
    Done,
}

impl<D: Driver> Interpreter<'_, D> {
    pub(super) fn execute(&mut self) {
        while let Ok(action) = self.step() {
            match action {
                Action::Enter {
                    place,
                    env,
                    return_names,
                } => {
                    trace!("enter new call frame");
                    if let Some(frame) = self.frames.last() {
                        self.frozen.insert(frame.place.name());
                    }

                    let frame = Frame {
                        place,
                        env,
                        return_names: Some(return_names),
                    };

                    self.frames.push(frame);
                }

                Action::Exit { return_values } => {
                    trace!("exit call frame");
                    let frame = self.frames.pop().unwrap();

                    if let Some(return_names) = frame.return_names {
                        assert_eq!(return_names.len(), return_values.len());

                        for (name, value) in return_names.into_iter().zip(return_values) {
                            self.bind(name, value);
                        }
                    }

                    if let Some(new_top) = self.frames.last() {
                        self.frozen.remove(&new_top.place.name());
                    }
                }

                Action::None => {}
            }
        }
    }

    fn step(&mut self) -> Result<Action, Step> {
        trace!("get current place");
        let place = self.get_place().ok_or(Step::Done)?;
        let op = self.get_operation(&place).unwrap(); // todo: figure out what to do here

        trace!("reduce args");
        let mut args = Vec::new();
        for arg in self.get_args(&op) {
            match self.reduce_value(&arg) {
                Some(value) => {
                    args.push(value);
                }

                None => {
                    args.push(self.locally_static_value(arg));
                }
            }
        }

        trace!("reduce operation");
        let targets = self.get_targets(&op);
        let result = self.reduce_op(op, args);

        if let Some(values) = result.values {
            trace!("binding results");
            assert_eq!(targets.len(), values.len());
            for (target, value) in targets.into_iter().zip(values) {
                self.bind(target, value);
            }
        }

        // During a call, the caller is "frozen", which means its body is
        // already undergoing partial evaluation, which means `self.redoing`
        // contains a kind of "partial" partially evaluated body (e.g. it might
        // contain the first half of the statements of this body). Freezing a
        // body prevents us from messing with this under, for instance,
        // recursive calls.
        if !self.frozen.contains(&place.name()) {
            if result.operation.is_some() {
                trace!("appending instruction");
            }

            match result.operation {
                Some(Operation::Branch(branch)) => {
                    let stmts = self.redoing.remove(&place.name()).unwrap_or_default();
                    let span = stmts
                        .iter()
                        .map(|stmt| stmt.span)
                        .reduce(Add::add)
                        .unwrap_or(branch.span)
                        + branch.span;
                    let ty = branch.ty;
                    let block = Block {
                        stmts,
                        branch,
                        span,
                        ty,
                    };

                    self.blocks.insert(place.name(), block);
                }

                Some(Operation::Statement(stmt)) => {
                    self.redoing.entry(place.name()).or_default().push(stmt);
                }

                None => {}
            }
        } else {
            trace!("function is frozen; skipping append");
        }

        trace!("moving forwards");
        self.next_op();

        Ok(result.action)
    }

    /// Move the interpreter to the next place. If the current step is a branch
    /// (like a return), then this will not do anything, under the assumption
    /// that `self.reduce_branch` will take care of emitting the correct
    /// action.
    fn next_op(&mut self) {
        let current = self.get_place().unwrap();

        match current {
            Place::Branch(_) => {}
            Place::Instruction(name, index) => {
                // UNWRAP: This function should never be called if we are not
                // currently in a block.
                let block = self.blocks.get(&name).unwrap();
                let new = if index < block.stmts.len() - 1 {
                    Place::Instruction(name, index + 1)
                } else {
                    Place::Branch(name)
                };

                self.get_frame_mut().place = new;
            }
        }
    }
}
