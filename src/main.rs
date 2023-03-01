mod lsp;
mod meta;
mod project;

use dashmap::DashMap;
use project::SourceName;
use simple_logger::SimpleLogger;
use zippy_common::source::Source;

fn main() {
    SimpleLogger::new().init().unwrap();
    lsp::lsp().unwrap();
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
