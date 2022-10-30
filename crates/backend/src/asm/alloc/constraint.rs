#[derive(Debug)]
pub struct Constraints {
    pub registers: &'static [RegisterInfo],
    pub call_clobbers: &'static [usize],
    pub parameters: &'static [usize],
    pub returns: &'static [usize],
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RegisterInfo {
    pub id: usize,
    pub size: usize,
    pub name: &'static str,
    pub aliases: &'static [usize],
}
