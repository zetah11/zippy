use std::collections::HashMap;

use super::{Type, UniVar};
use crate::names::{Actual, Name, Names};

pub fn pretty_type(
    names: &Names,
    subst: &HashMap<UniVar, &Type>,
    map: &mut PrettyMap,
    ty: &Type,
) -> String {
    let mut prettier = Prettier::new(names, subst, map);
    prettier.pretty(ty)
}

#[derive(Debug, Default)]
pub struct PrettyMap {
    unbound: HashMap<UniVar, String>,
    curr: usize,
}

impl PrettyMap {
    pub fn new() -> Self {
        Self {
            unbound: HashMap::new(),
            curr: 0,
        }
    }

    const ALPHABET: &[char] = &[
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];

    fn var(&mut self, var: &UniVar) -> String {
        if let Some(name) = self.unbound.get(var) {
            return name.clone();
        }

        let mut id = self.curr;
        self.curr += 1;

        let mut name = String::with_capacity(2);
        name.push('\'');

        if id == 0 {
            name.push('a');
        } else {
            let n = Self::ALPHABET.len();
            while id != 0 {
                let ch = Self::ALPHABET[id % n];
                name.insert(1, ch);
                id /= n;
            }
        }

        self.unbound.entry(*var).or_insert(name).clone()
    }
}

struct Prettier<'a> {
    names: &'a Names,
    subst: &'a HashMap<UniVar, &'a Type>,
    insts: Vec<HashMap<Name, Type>>,
    map: &'a mut PrettyMap,
}

impl<'a> Prettier<'a> {
    pub fn new(
        names: &'a Names,
        subst: &'a HashMap<UniVar, &'a Type>,
        map: &'a mut PrettyMap,
    ) -> Self {
        Self {
            names,
            subst,
            insts: Vec::new(),
            map,
        }
    }

    pub fn pretty(&mut self, ty: &Type) -> String {
        self.pretty_type(ty)
    }

    fn get(&self, name: &Name) -> Option<&Type> {
        for inst in self.insts.iter().rev() {
            if let Some(ty) = inst.get(name) {
                return Some(ty);
            }
        }

        None
    }

    fn var(&mut self, var: &UniVar) -> String {
        self.map.var(var)
    }

    fn push(&mut self, inst: HashMap<Name, Type>) {
        self.insts.push(inst);
    }

    fn pop(&mut self) {
        assert!(self.insts.pop().is_some());
    }

    fn pretty_type(&mut self, ty: &Type) -> String {
        match ty {
            Type::Instantiated(ty, inst) => {
                self.push(inst.clone());
                let res = self.pretty_base(ty);
                self.pop();
                res
            }

            Type::Var(_, var) => {
                if let Some(ty) = self.subst.get(var) {
                    self.pretty_type(ty)
                } else {
                    self.var(var)
                }
            }

            ty => self.pretty_arrow(ty),
        }
    }

    fn pretty_arrow(&mut self, ty: &Type) -> String {
        match ty {
            Type::Fun(t, u) => {
                let t = self.pretty_range(t);
                let u = self.pretty_range(u);

                format!("{t} -> {u}")
            }

            Type::Var(_, var) => {
                if let Some(ty) = self.subst.get(var) {
                    self.pretty_arrow(ty)
                } else {
                    self.var(var)
                }
            }

            ty => self.pretty_range(ty),
        }
    }

    fn pretty_range(&mut self, ty: &Type) -> String {
        match ty {
            Type::Range(lo, hi) => {
                if *lo == 0 {
                    format!("{hi}")
                } else {
                    format!("{lo} upto {hi}")
                }
            }

            Type::Var(_, var) => {
                if let Some(ty) = self.subst.get(var) {
                    self.pretty_range(ty)
                } else {
                    self.var(var)
                }
            }

            ty => self.pretty_product(ty),
        }
    }

    fn pretty_product(&mut self, ty: &Type) -> String {
        match ty {
            Type::Product(t, u) => {
                let t = self.pretty_base(t);
                let u = self.pretty_product(u);

                format!("{t} * {u}")
            }

            Type::Var(_, var) => {
                if let Some(ty) = self.subst.get(var) {
                    self.pretty_product(ty)
                } else {
                    self.var(var)
                }
            }

            ty => self.pretty_base(ty),
        }
    }

    fn pretty_base(&mut self, ty: &Type) -> String {
        match ty {
            Type::Name(name) => {
                if let Some(ty) = self.get(name) {
                    self.pretty_base(&ty.clone())
                } else {
                    match &self.names.get_path(name).1 {
                        Actual::Lit(name) => name.clone(),
                        Actual::Generated(name) => name.to_string("T"),
                        _ => unreachable!(),
                    }
                }
            }

            Type::Var(_, var) => {
                if let Some(ty) = self.subst.get(var) {
                    self.pretty_base(ty)
                } else {
                    self.var(var)
                }
            }

            Type::Number => "<number>".into(),
            Type::Type => "type".into(),

            Type::Invalid => "<error>".into(),

            ty => format!("({})", self.pretty_type(ty)),
        }
    }
}
