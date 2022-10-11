mod check;
mod irreducible;
mod promote;
mod reduce;

use im::HashMap;

use crate::message::Messages;
use crate::mir::pretty::Prettier;
use crate::mir::{Context, Decls, Types, ValueDef};
use crate::resolve::names::{Name, Names};
use crate::Driver;

use self::irreducible::{Irreducible, IrreducibleNode};

pub fn evaluate(
    driver: &mut impl Driver,
    context: &mut Context,
    names: &mut Names,
    types: &Types,
    decls: Decls,
) -> Decls {
    let (res, messages) = {
        let mut lowerer = Lowerer::new(driver, context, names, types);
        let res = lowerer.reduce_decls(decls);

        (res, lowerer.messages)
    };

    driver.report(messages);

    res
}

#[derive(Clone, Debug, Default)]
struct Env {
    map: HashMap<Name, Irreducible>,
    parent: Option<Box<Env>>,
}

impl Env {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            parent: None,
        }
    }

    fn child(&self) -> Self {
        Self {
            map: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    fn lookup(&self, name: &Name) -> Option<&Irreducible> {
        self.map
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.lookup(name)))
    }

    fn set(&mut self, name: Name, value: Irreducible) {
        self.map.insert(name, value);
    }

    /// Create a child environment where the given name is bound to the given value.
    fn with(&self, name: Name, value: Irreducible) -> Self {
        Self {
            map: self.map.update(name, value),
            parent: Some(Box::new(self.clone())),
        }
    }
}

#[derive(Debug)]
struct Lowerer<'a, Driver> {
    env: Env,
    names: &'a mut Names,
    types: &'a Types,
    context: &'a mut Context,

    driver: &'a mut Driver,
    messages: Messages,
}

impl<'a, D: Driver> Lowerer<'a, D> {
    fn new(
        driver: &'a mut D,
        context: &'a mut Context,
        names: &'a mut Names,
        types: &'a Types,
    ) -> Self {
        Self {
            env: Env::new(),
            names,
            types,
            context,

            driver,
            messages: Messages::new(),
        }
    }

    fn reduce_decls(&mut self, decls: Decls) -> Decls {
        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values {
            self.driver.report_eval({
                let prettier = Prettier::new(self.names, self.types);
                prettier.pretty_name(&def.name)
            });

            let bind = self.reduce_exprs(self.env.clone(), def.bind);
            self.env.set(def.name, bind.clone());
            values.push(ValueDef {
                name: def.name,
                span: def.span,
                bind: self.promote(bind),
            });
        }

        self.driver.done_eval();

        Decls { values }
    }
}
