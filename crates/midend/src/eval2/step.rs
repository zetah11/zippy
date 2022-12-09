use std::ops::Add;

use common::mir::Block;

use super::action::Action;
use super::place::Place;
use super::value::Operation;
use super::{Interpreter, ReducedValue};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Step {
    /// The interpreter has nothing more to do.
    Done,
}

impl Interpreter<'_> {
    pub(super) fn step(&mut self) -> Result<Action, Step> {
        let place = self.get_place().ok_or(Step::Done)?;
        let op = self.get_operation(&place).unwrap(); // todo: figure out what to do here

        let mut args = Vec::new();
        for arg in self.get_args(&op) {
            match self.reduce_value(&arg) {
                Some(value) => {
                    args.push(value);
                }

                None => {
                    args.push(ReducedValue::Static(arg));
                }
            }
        }

        let targets = self.get_targets(&op);
        let result = self.reduce_op(op, args);

        if let Some(values) = result.values {
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
        }

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
                    Place::Branch(name)
                } else {
                    Place::Instruction(name, index + 1)
                };

                self.get_frame_mut().place = new;
            }
        }
    }
}
