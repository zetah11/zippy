mod action;
mod environment;
mod place;
mod reduce;
mod state;
mod step;
mod value;

use std::collections::{HashMap, HashSet};

use common::mir::{Block, Context, Decls, Statement, Types, Value};
use common::names::{Name, Names};

use self::action::Action;
use self::state::Frame;
use self::value::ReducedValue;

pub fn evaluate(
    context: &Context,
    names: &Names,
    types: &Types,
    entry: Option<Name>,
    decls: Decls,
) {
    let mut interp = Interpreter::new(context, names, types, decls);
    if let Some(name) = entry {
        interp.entry(name);
    }

    interp.run();
}

#[derive(Debug)]
struct Interpreter<'a> {
    context: &'a Context,
    names: &'a Names,
    types: &'a Types,
    decls: Decls,

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
            names,
            types,
            decls,

            frames: Vec::new(),
            globals: HashMap::new(),

            blocks: HashMap::new(),
            functions: HashMap::new(),
            redoing: HashMap::new(),
            frozen: HashSet::new(),
        }
    }

    pub fn entry(&mut self, name: Name) {
        let place = self.place_of(&name).unwrap();
        let frame = Frame::new(place);
        self.frames.push(frame);
    }

    pub fn run(&mut self) {
        while let Ok(action) = self.step() {
            match action {
                Action::Enter {
                    place,
                    env,
                    return_names,
                } => {
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
}
