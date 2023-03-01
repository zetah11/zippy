mod lsp;
mod meta;
mod project;

use dashmap::DashMap;
use project::SourceName;
use zippy_common::source::Source;

fn main() {
    let mut args = std::env::args();
    let program_name = match args.next() {
        Some(arg) => arg,
        None => print_usage_info("zc"),
    };

    match args.next().as_ref().map(AsRef::as_ref) {
        Some("lsp") => lsp::lsp().unwrap(),
        _ => print_usage_info(&program_name),
    }
}

#[salsa::db(zippy_common::Jar, zippy_frontend::Jar)]
struct Database {
    storage: salsa::Storage<Self>,
    sources: DashMap<SourceName, Source>,
}

impl salsa::Database for Database {}

impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Self {
            storage: self.storage.snapshot(),
            sources: DashMap::new(),
        })
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            storage: salsa::Storage::default(),
            sources: DashMap::new(),
        }
    }

    /// Add a source to the database, as long as it hasn't been added yet.
    pub fn add_source(&self, name: SourceName, content: String) -> Source {
        let source = Source::new(self, name.as_path().to_path_buf(), content);
        assert!(self.sources.insert(name, source).is_none());
        source
    }

    /// Get the content of the source with the given name.
    pub fn read_source(&self, source: SourceName) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(source.as_path())?)
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
    eprintln!(
        "  lsp   run {} as a language server on stdio",
        meta::COMPILER_NAME
    );

    // eugh whatever
    std::process::exit(1)
}
