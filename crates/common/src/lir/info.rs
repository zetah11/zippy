use std::collections::HashMap;
use std::fmt::{self, Display};

use bitflags::bitflags;

use crate::names::Name;

bitflags! {
    pub struct Info: u64 {
        const PROCEDURE = 0b01;
        const EXTERN    = 0b10;
    }
}

impl Info {
    pub fn procedure() -> Self {
        Self::PROCEDURE
    }

    pub fn constant() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CallingConvention {
    #[default]
    Corollary,
    SystemV,
    Stdcall,
}

impl Display for CallingConvention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corollary => write!(f, "corollary"),
            Self::SystemV => write!(f, "systemv"),
            Self::Stdcall => write!(f, "stdcall"),
        }
    }
}

#[derive(Debug, Default)]
pub struct NameInfo {
    map: HashMap<Name, Info>,
    conventions: HashMap<Name, CallingConvention>,
}

impl NameInfo {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            conventions: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: Name, info: Info) {
        assert!(self.map.insert(name, info).is_none());
    }

    pub fn add_convention(&mut self, name: Name, convention: CallingConvention) {
        assert!(self.map.contains_key(&name));
        assert!(self.conventions.insert(name, convention).is_none());
    }

    pub fn get(&self, name: &Name) -> Option<Info> {
        self.map.get(name).copied()
    }

    pub fn get_convention(&self, name: &Name) -> Option<CallingConvention> {
        if self.is_procedure(name) {
            Some(self.conventions.get(name).copied().unwrap_or_default())
        } else {
            None
        }
    }

    pub fn is(&self, name: &Name, info: Info) -> bool {
        self.get(name).map(|i| i.contains(info)).unwrap_or(false)
    }

    pub fn is_constant(&self, name: &Name) -> bool {
        !self.is_procedure(name)
    }

    pub fn is_extern(&self, name: &Name) -> bool {
        self.is(name, Info::EXTERN)
    }

    pub fn is_intern(&self, name: &Name) -> bool {
        !self.is_extern(name)
    }

    pub fn is_procedure(&self, name: &Name) -> bool {
        self.is(name, Info::PROCEDURE)
    }
}
