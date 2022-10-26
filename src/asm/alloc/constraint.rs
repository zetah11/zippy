#[derive(Debug)]
pub struct Constraints {
    pub registers: &'static [RegisterInfo],
    pub call_clobbers: &'static [usize],
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RegisterInfo {
    pub size: usize,
    pub name: &'static str,
    pub aliases: &'static [usize],
}
