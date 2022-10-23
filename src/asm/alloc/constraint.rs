#[derive(Debug)]
pub struct Constraints {
    pub registers: &'static [RegisterInfo],
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RegisterInfo {
    pub size: usize,
    pub name: &'static str,
}
