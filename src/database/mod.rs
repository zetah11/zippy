use std::collections::HashMap;
use std::path::PathBuf;

use bimap::BiMap;
use dashmap::DashMap;
use zippy_common::names::ItemName;
use zippy_common::source::project::module_name_from_source;
use zippy_common::source::{Module, Source, SourceName};

#[derive(Default)]
#[salsa::db(zippy_common::Jar, zippy_frontend::Jar)]
pub struct Database {
    storage: salsa::Storage<Self>,

    // Inputs
    source_names: BiMap<PathBuf, SourceName>,
    sources: DashMap<SourceName, Source>,
    modules: DashMap<ItemName, Module>,
}

impl salsa::Database for Database {}

impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Self {
            storage: self.storage.snapshot(),

            source_names: self.source_names.clone(),
            sources: self.sources.clone(),
            modules: self.modules.clone(),
        })
    }
}

impl zippy_common::Db for Database {
    fn get_module(&self, name: &ItemName) -> Option<Module> {
        self.modules.get(name).map(|module| *module)
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            storage: salsa::Storage::default(),

            source_names: BiMap::new(),
            sources: DashMap::new(),
            modules: DashMap::new(),
        }
    }

    /// Initialize this database with some sources.
    pub fn init_sources(&mut self, sources: Vec<(PathBuf, SourceName, String)>) {
        let mut modules: HashMap<_, Vec<_>> = HashMap::new();

        for (path, name, content) in sources {
            let source = Source::new(self, name, content);
            assert!(!self.source_names.insert(path, name).did_overwrite());
            assert!(self.sources.insert(name, source).is_none());

            let module_name = module_name_from_source(self, name);
            modules.entry(module_name).or_default().push(source);
        }

        for (name, sources) in modules {
            let module = Module::new(self, name, sources);
            assert!(self.modules.insert(name, module).is_none());
        }
    }

    /// Update the contents of a source file, creating it if it does not exist.
    pub fn update_source(&mut self, path: PathBuf, name: SourceName, content: String) {
        let source = self.sources.get(&name).map(|source| *source);

        if let Some(source) = source {
            assert_eq!(Some(&path), self.source_names.get_by_right(&name));
            source.set_content(self).to(content);
            return;
        }

        let source = Source::new(self, name, content);
        assert!(!self.source_names.insert(path, name).did_overwrite());
        self.sources.insert(name, source);

        let module_name = module_name_from_source(self, name);

        let module = self.modules.get(&module_name).map(|module| *module);

        if let Some(module) = module {
            let mut sources = module.sources(self).clone();
            sources.push(source);
            module.set_sources(self).to(sources);
            return;
        }

        let module = Module::new(self, module_name, vec![source]);
        self.modules.insert(module_name, module);
    }

    /// Iterate over every module in this database.
    pub fn get_modules(&self) -> impl Iterator<Item = Module> + '_ {
        self.modules.iter().map(|module| *module)
    }

    /// Get a source by name, if it exists.
    pub fn get_source(&self, name: &SourceName) -> Option<Source> {
        self.sources.get(name).map(|source| *source)
    }

    /// Get the path corresponding to this source name.
    pub fn get_source_path(&self, name: &SourceName) -> &PathBuf {
        self.source_names
            .get_by_right(name)
            .expect("source name with no associated path")
    }

    /// Get the name name corresponding to a specific path, or create one if it
    /// doesn't exist.
    pub fn with_source_name(
        &self,
        path: PathBuf,
        f: impl FnOnce(PathBuf) -> SourceName,
    ) -> SourceName {
        match self.source_names.get_by_left(&path) {
            Some(name) => *name,
            None => f(path),
        }
    }
}
