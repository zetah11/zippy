use std::path::PathBuf;

use clap::{ArgAction, Parser};

/// a smol functional/imperative programming language.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
pub struct Arguments {
    /// Completely disable partial evaluation.
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_eval: bool,

    /// Never overwrite lines in the compiler output.
    #[arg(long, action = ArgAction::SetTrue)]
    pub preserve_output: bool,

    #[arg(required = true)]
    pub path: PathBuf,
}
