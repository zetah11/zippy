mod action;
mod discover;
mod environment;
mod place;
mod reduce;
mod state;
mod step;
mod value;

use std::collections::{HashMap, HashSet};

use common::mir::pretty::Prettier;
use common::mir::{
    Block, Branch, BranchNode, Context, Decls, Statement, StaticValue, StaticValueNode, Types,
    Value, ValueNode,
};
use common::names::{Name, Names};
use common::Driver;
use log::{info, trace};

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
