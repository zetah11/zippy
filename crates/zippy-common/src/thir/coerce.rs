use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CoercionId(usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Coercion {
    Upcast,
}

#[derive(Debug, Default)]
pub struct Coercions {
    coercions: HashMap<CoercionId, Coercion>,
    curr: usize,
}

impl Coercions {
    pub fn new() -> Self {
        Self {
            coercions: HashMap::new(),
            curr: 0,
        }
    }

    pub fn fresh(&mut self) -> CoercionId {
        let id = CoercionId(self.curr);
        self.curr += 1;
        id
    }

    pub fn add(&mut self, id: CoercionId, data: Coercion) {
        assert!(self.coercions.insert(id, data).is_none());
    }

    pub fn get(&self, id: &CoercionId) -> Option<Coercion> {
        self.coercions.get(id).copied()
    }
}
