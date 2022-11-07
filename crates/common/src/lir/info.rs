use std::collections::HashMap;

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

#[derive(Debug, Default)]
pub struct NameInfo {
    map: HashMap<Name, Info>,
}

impl NameInfo {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: Name, info: Info) {
        assert!(self.map.insert(name, info).is_none());
    }

    pub fn get(&self, name: &Name) -> Option<Info> {
        self.map.get(name).copied()
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
