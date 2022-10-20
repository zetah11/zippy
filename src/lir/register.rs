use super::TypeId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Virtual {
    pub id: usize,
    pub ty: TypeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Register {
    Virtual {
        reg: Virtual,
        ndx: Option<(usize, TypeId)>,
    },

    Frame(usize, TypeId),
    Physical(usize),
}
