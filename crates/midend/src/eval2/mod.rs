mod code;
mod env;
mod interpreter;
mod result;
mod state;

use common::mir::{Decls, Types};
use common::names::{Name, Names};

use self::interpreter::Interpreter;

pub fn evaluate(names: &Names, types: &Types, entry: Option<Name>, decls: Decls) {
    let mut interp = Interpreter::new(names, types, decls);
    if let Some(name) = entry {
        interp.entry(name);
    }

    interp.run().unwrap();
    println!("{:?}", interp.returned());
}
