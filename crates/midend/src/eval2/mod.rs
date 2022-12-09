mod action;
mod discover;
mod environment;
mod place;
mod reduce;
mod state;
mod step;
mod value;

use std::collections::{HashMap, HashSet};

use common::mir::{
    Block, Branch, BranchNode, Context, Decls, Statement, StaticValue, StaticValueNode, Types,
    Value, ValueNode,
};
use common::names::{Name, Names};

use self::state::Frame;
use self::value::ReducedValue;

pub fn evaluate(
    context: &Context,
    names: &Names,
    types: &Types,
    entry: Option<Name>,
    decls: Decls,
) -> Decls {
    let mut interp = Interpreter::new(context, names, types, decls);
    interp.discover(entry);
    interp.run();
    interp.collect()
}

#[derive(Debug)]
struct Interpreter<'a> {
    context: &'a Context,
    _names: &'a Names,
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

impl<'a> Interpreter<'a> {
    pub fn new(context: &'a Context, names: &'a Names, types: &'a Types, decls: Decls) -> Self {
        Self {
            context,
            _names: names,
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
            self.discover_all();
        }
    }

    pub fn run(&mut self) {
        while let Some(name) = self.worklist.pop() {
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
