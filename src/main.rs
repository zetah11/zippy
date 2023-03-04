mod cli;
mod lsp;
mod meta;
mod output;
mod pretty;
mod project;
mod span;

use std::path::PathBuf;

use bimap::BiMap;
use dashmap::DashMap;
use zippy_common::names::ItemName;
use zippy_common::source::{Source, SourceName};
use zippy_frontend::ast;

fn main() {
    let mut args = std::env::args();
    let program_name = match args.next() {
        Some(arg) => arg,
        None => print_usage_info("zc"),
    };

    let result = match args.next().as_ref().map(AsRef::as_ref) {
        Some("check") => cli::check(),
        Some("lsp") => lsp::lsp(),
        _ => print_usage_info(&program_name),
    };

    result.unwrap()
}

#[derive(Default)]
#[salsa::db(zippy_common::Jar, zippy_frontend::Jar)]
pub struct Database {
    storage: salsa::Storage<Self>,
    root: Option<PathBuf>,

    // Inputs
    source_names: BiMap<PathBuf, SourceName>,
    sources: DashMap<SourceName, Source>,

    ast_modules: DashMap<ItemName, ast::Module>,
}

impl salsa::Database for Database {}

impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Self {
            storage: self.storage.snapshot(),
            root: self.root.clone(),

            source_names: self.source_names.clone(),
            sources: self.sources.clone(),

            ast_modules: self.ast_modules.clone(),
        })
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            storage: salsa::Storage::default(),
            root: None,

            source_names: BiMap::new(),
            sources: DashMap::new(),

            ast_modules: DashMap::new(),
        }
    }

    pub fn with_root(self, root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
            ..self
        }
    }

    /// Write the given content to a source file with the given path and source
    /// name. This should only be called once times for the same file.
    pub fn write_source(&mut self, path: PathBuf, name: SourceName, content: String) {
        let source = Source::new(self, name, content);

        assert!(!self.source_names.insert(path, name).did_overwrite());
        assert!(self.sources.insert(name, source).is_none());
    }
}

fn print_usage_info(program_name: &str) -> ! {
    eprintln!(
        "{} version {}-{}",
        meta::COMPILER_NAME,
        meta::VERSION,
        meta::TAG
    );
    eprintln!("usage: {} <command>", program_name);
    eprintln!();
    eprintln!("available commands:");
    eprintln!("  check    check the project for errors");
    eprintln!(
        "  lsp      run {} as a language server on stdio",
        meta::COMPILER_NAME
    );

    // eugh whatever
    std::process::exit(1)
}
