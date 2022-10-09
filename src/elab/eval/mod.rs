mod bind;
mod reduce;

use std::collections::HashMap;

use log::{info, trace};

use crate::message::Messages;
use crate::mir::pretty::Prettier;
use crate::mir::{Decls, Expr, ValueDef};
use crate::resolve::names::{Name, Names};
use crate::Driver;

pub fn evaluate(driver: &mut impl Driver, names: &mut Names, decls: Decls) -> Decls {
    info!("beginning evaluation");

    let (res, messages) = {
        let mut evaler = Evaluator::new(names, driver);
        let res = evaler.reduce_decls(decls);
        (res, evaler.messages)
    };

    driver.done_eval();
    driver.report(messages);

    trace!("done evaluating");

    res
}

#[derive(Debug)]
pub struct Evaluator<'a, Driver> {
    context: (HashMap<Name, Expr>, Vec<HashMap<Name, Expr>>),
    messages: Messages,
    driver: &'a mut Driver,
    names: &'a mut Names,
}

impl<'a, D: Driver> Evaluator<'a, D> {
    pub fn new(names: &'a mut Names, driver: &'a mut D) -> Self {
        Self {
            context: (HashMap::new(), Vec::new()),
            messages: Messages::new(),
            driver,
            names,
        }
    }

    /// Reduce the given declarations by partially evaluating the non-effectful code.
    pub fn reduce_decls(&mut self, decls: Decls) -> Decls {
        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values.iter() {
            self.bind(def.pat.clone(), def.bind.clone());
        }

        for def in decls.values {
            let ValueDef { span, pat, bind } = def;

            self.driver.report_eval({
                let prettier = Prettier::new(self.names);
                prettier.pretty_pat(&pat)
            });

            self.enter();
            let bind = self.reduce(bind);
            self.exit();
            self.bind(pat.clone(), bind.clone());
            values.push(ValueDef { span, pat, bind });
        }

        Decls { values }
    }

    /// Reduce the given expression by partially evaluating its non-effectful sub-expressions.
    pub fn reduce_expr(&mut self, expr: Expr) -> Expr {
        self.reduce(expr)
    }

    fn enter(&mut self) {
        self.context.1.push(HashMap::new());
    }

    fn exit(&mut self) {
        assert!(self.context.1.pop().is_some());
    }

    fn lookup(&mut self, name: &Name) -> Option<&Expr> {
        for ctx in self.context.1.iter().rev() {
            if let Some(v) = ctx.get(name) {
                return Some(v);
            }
        }

        self.context.0.get(name)
    }

    fn set(&mut self, name: Name, value: Expr) {
        if let Some(map) = self.context.1.last_mut() {
            map.insert(name, value);
        } else {
            self.context.0.insert(name, value);
        }
    }
}
