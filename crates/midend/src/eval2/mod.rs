mod code;
mod env;
mod interpreter;
mod result;
mod state;

use common::mir::Decls;
use common::names::Name;

use self::interpreter::Interpreter;

pub fn evaluate(entry: Option<Name>, decls: Decls) {
    let mut interp = Interpreter::new(decls);
    if let Some(name) = entry {
        interp.entry(name);
    }

    interp.run().unwrap();
    println!("{:?}", interp.returned());
}
