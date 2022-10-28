use super::TypeId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Virtual {
    pub id: usize,
    pub ty: TypeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Register {
    Virtual(Virtual),
    Frame(isize, TypeId),
    Physical(usize),
}