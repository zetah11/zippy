use std::collections::HashMap;

use super::kinds::{Kind, UniVar};
use super::names::Namer;

pub struct Prettier<'a> {
    subst: &'a HashMap<UniVar, Kind>,
}

impl<'a> Prettier<'a> {
    pub fn new(subst: &'a HashMap<UniVar, Kind>) -> Self {
        Self { subst }
    }

    pub fn pretty(&self, namer: &mut Namer, mut kind: &'a Kind) -> String {
        loop {
            return match kind {
                Kind::Type => "type".into(),
                Kind::Invalid => "<error>".into(),

                Kind::Function(a, b) => {
                    let a = self.pretty(namer, a);
                    let b = self.pretty(namer, b);
                    format!("{a} -> {b}")
                }

                Kind::Product(a, b) => {
                    let a = self.pretty(namer, a);
                    let b = self.pretty(namer, b);
                    format!("{a} * {b}")
                }

                Kind::Var(var) => {
                    if let Some(subst) = self.subst.get(var) {
                        kind = subst;
                        continue;
                    } else {
                        namer.pretty(*var)
                    }
                }
            };
        }
    }
}

/*
    pub fn pretty<'a>(&'a self, namer: &mut Namer, mut kind: &'a Kind) -> String {
    }
*/
