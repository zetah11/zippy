use super::TypeId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Virtual {
    pub id: usize,
    pub ty: TypeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Register {
    Virtual(Virtual),
    Frame(BaseOffset, TypeId),
    Physical(usize),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BaseOffset {
    Local(usize),
    Argument { offset: usize, total: usize },
    Parameter { offset: usize, total: usize },
}
