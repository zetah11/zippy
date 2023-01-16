use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};

/// a smol functional/imperative programming language.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
#[command(propagate_version = true)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    #[arg(last = true)]
    pub options: Vec<String>,
}

impl Arguments {
    pub fn options(&self) -> &Options {
        self.command.options()
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(visible_alias = "r")]
    Run(Options),
    #[command(visible_alias = "b")]
    Build(Options),
    #[command(visible_alias = "c")]
    Check(Options),

    #[command()]
    Clean(Options),
}

impl Command {
    pub fn options(&self) -> &Options {
        match self {
            Self::Run(opts) => opts,
            Self::Build(opts) => opts,
            Self::Check(opts) => opts,
            Self::Clean(opts) => opts,
        }
    }

    pub fn run(&self) -> bool {
        matches!(self, Self::Run(_))
    }

    pub fn build(&self) -> bool {
        matches!(self, Self::Run(_) | Self::Build(_))
    }

    pub fn check(&self) -> bool {
        matches!(self, Self::Run(_) | Self::Build(_) | Self::Check(_))
    }
}

#[derive(Debug, Args)]
pub struct Options {
    /// Completely disable partial evaluation.
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_eval: bool,

    /// Never overwrite lines in the compiler output.
    #[arg(long, action = ArgAction::SetTrue)]
    pub preserve_output: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub output_ir: bool,

    #[arg(long, default_value = "artifacts")]
    pub artifacts: PathBuf,

    /// The target to compile the code for.
    #[arg(short, long)]
    pub target: Option<String>,

    #[arg(required = true)]
    pub path: PathBuf,
}
