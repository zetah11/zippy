pub use tree::{Expr, ExprNode, Pat, PatNode, Type};

mod bind;
mod check;
mod context;
mod infer;
mod lower;
mod solve;
mod tree;

use crate::message::Messages;
use crate::resolve::names::Name;
use crate::{hir, Driver};
use context::Context;
use lower::lower;

pub fn typeck(driver: &mut impl Driver, ex: hir::Expr<Name>) -> Expr<Type> {
    let ex = lower(ex);
    let mut typer = Typer::new();
    let ex = typer.infer(ex);

    driver.report(typer.messages);

    ex
}

#[derive(Debug, Default)]
struct Typer {
    pub messages: Messages,
    context: Context,
}

impl Typer {
    pub fn new() -> Self {
        Self {
            messages: Messages::new(),
            context: Context::new(),
        }
    }
}
