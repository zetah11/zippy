pub mod x64;

use std::error::Error;
use std::fmt::{self, Display};

use target_lexicon::Triple;

#[derive(Debug)]
pub enum CodegenError {
    TargetNotSupported(Triple),
}

impl Error for CodegenError {}
impl Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetNotSupported(triple) => write!(f, "the target '{triple}' is unsupported"),
        }
    }
}
