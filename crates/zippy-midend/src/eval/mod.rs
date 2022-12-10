//! # Partial evaluation
//!
//! This module implements the partial evaluator. Its main job is to evaluate
//! all of the pure fragments of a program. It's called a partial evaluator
//! because it needs to do its job even when we only know parts of the input
//! data. For example, it may try to evaluate a function without any knowledge
//! of what its parameters will contain. This means that the partial evaluator
//! will sometimes reduce a call or a function body down to a single value,
//! while it may other times be unable to do anything. When it is able to reduce
//! and simplify functions and blocks, it will "re-write" them to the more
//! simplified case (in the extreme case, to a single `return`). If it can't
//! make any progress, the block will be left as is.
//!
//! This partial evaluator is vaguely modelled after rustc's const evaluator.
//! The bird's eye view is that we have a function `execute` which repeatedly
//! calls a `step` function until there is no more to do. `step` will fetch the
//! current instruction, reduce its arguments as best it can, evaluate the
//! actual operation, and then bind the resulting values to any names. When
//! evaluating the operation, it might not be able to do anything, in which case
//! it will emit a (possibly reduced) copy of itself. When we have fully
//! evaluated all the instructions (statements and branch) in a block, it
//! collects all the emitted instructions, and replaces the current block with
//! them.
//!
//! Because it is doing this rewriting at the same time as we are evaluating
//! things, some care needs to be applied to make sure we keep track of what is
//! "static" data and what is "dynamic" data. Take for instance this program:
//!
//! ```zippy
//! fun id (x: 10) = x
//! fun first (x: 10, y: 10) = x
//! fun main (?: 1) = first (id 5, id 6)
//! ```
//!
//! The program calls `id` twice, with two different arguments. If we "naively"
//! replaced the variable `x` in its body, we might incorrectly reducing it to
//!
//! ```zippy
//! fun id (x: 10) = 6
//! ```
//!
//! which is of course wrong. Because of this, values also keep track of a
//! *frame index*, which says where a value originated, which lets us decide
//! whether or not to use the reduced or unreduced form of a value while
//! rewriting.

mod action;
mod discover;
mod environment;
mod place;
mod reduce;
mod state;
mod step;
mod value;

use std::collections::{HashMap, HashSet};

use log::{info, trace};
use zippy_common::mir::pretty::Prettier;
use zippy_common::mir::{
    Block, Branch, BranchNode, Context, Decls, Statement, StaticValue, StaticValueNode, Types,
    Value, ValueNode,
};
use zippy_common::names::{Name, Names};
use zippy_common::Driver;

use self::state::Frame;

pub fn evaluate(
    driver: &mut impl Driver,
    context: &Context,
    names: &Names,
    types: &Types,
    entry: Option<Name>,
    decls: Decls,
) -> Decls {
    info!("beginning evaluation");

    let mut interp = Interpreter::new(driver, context, names, types, decls);
    interp.discover(entry);
    trace!("discovery done");

    interp.run();
    trace!("execution done");

    let res = interp.collect();
    trace!("evaluation done");

    driver.done_eval();

    res
}

#[derive(Debug)]
struct Interpreter<'a, D> {
    driver: &'a mut D,

    context: &'a Context,
    names: &'a Names,
    types: &'a Types,
    decls: Decls,

    worklist: Vec<Name>,
    frames: Vec<Frame>,
    globals: HashMap<Name, Value>,

    blocks: HashMap<Name, Block>,
    functions: HashMap<Name, Vec<Name>>,
    redoing: HashMap<Name, Vec<Statement>>,
    frozen: HashSet<Name>,
}

impl<'a, D: Driver> Interpreter<'a, D> {
    pub fn new(
        driver: &'a mut D,
        context: &'a Context,
        names: &'a Names,
        types: &'a Types,
        decls: Decls,
    ) -> Self {
        Self {
            driver,

            context,
            names,
            types,
            decls,

            worklist: Vec::new(),
            frames: Vec::new(),
            globals: HashMap::new(),

            blocks: HashMap::new(),
            functions: HashMap::new(),
            redoing: HashMap::new(),
            frozen: HashSet::new(),
        }
    }

    pub fn discover(&mut self, entry: Option<Name>) {
        if let Some(entry) = entry {
            self.discover_from_entry(entry);
        } else {
            trace!("no entry point; discovering everything");
            self.discover_all();
        }
    }

    pub fn run(&mut self) {
        while let Some(name) = self.worklist.pop() {
            let at = {
                let prettier = Prettier::new(self.names, self.types);
                prettier.pretty_name(&name)
            };

            trace!(
                "eval top-level; {} name{} left",
                self.worklist.len() + 1,
                if self.worklist.is_empty() { "" } else { "s" }
            );

            self.driver.report_eval(at);

            let place = self.place_of(&name).unwrap();
            let frame = Frame::new(place);
            self.frames.push(frame);

            self.execute();
        }
    }

    pub fn collect(mut self) -> Decls {
        let mut res = Decls::new(Vec::new());

        for (name, value) in self.globals {
            let Value { span, ty, .. } = value;

            let node = match value.node {
                ValueNode::Int(i) => StaticValueNode::Int(i),
                _ => {
                    let node = BranchNode::Return(vec![value]);
                    let branch = Branch { node, span, ty };

                    let block = Block::new(span, ty, Vec::new(), branch);
                    StaticValueNode::LateInit(block)
                }
            };

            let value = StaticValue { node, span, ty };

            res.values.insert(name, value);
        }

        for (name, block) in self.blocks {
            if let Some(params) = self.functions.remove(&name) {
                res.functions.insert(name, (params, block));
            } else {
                res.values.entry(name).or_insert_with(|| {
                    let Block { span, ty, .. } = block;
                    let node = StaticValueNode::LateInit(block);
                    StaticValue { node, span, ty }
                });
            };
        }

        res
    }
}
