use dashmap::DashMap;
use zc::source::{SourceGen, SourceId};

/// A thread-safe, bijective map between filenames and source ids.
#[derive(Debug, Default)]
pub struct FilenameMap {
    name_to_id: DashMap<String, SourceId>,
    id_to_name: DashMap<SourceId, String>,
    id_gen: SourceGen,
}

impl FilenameMap {
    /// Create a new empty filename <-> id map.
    pub fn new() -> Self {
        Self {
            name_to_id: DashMap::new(),
            id_to_name: DashMap::new(),
            id_gen: SourceGen::new(),
        }
    }

    /// Add a new name to the map, and return its unique, generated id.
    ///
    /// # Panics
    ///
    /// Panics if the name already exists in the map.
    pub fn add(&self, name: String) -> SourceId {
        let id = self.id_gen.new_id();

        assert!(self.name_to_id.insert(name.clone(), id).is_none());
        self.id_to_name.insert(id, name);

        id
    }

    /// Get the id assosciated with the given name.
    pub fn get_id(&self, name: &String) -> Option<SourceId> {
        self.name_to_id.get(name).map(|id| *id)
    }

    /// Get the name assosciated with the given id.
    pub fn get_name(&self, id: &SourceId) -> Option<String> {
        self.id_to_name.get(id).map(|name| name.clone())
    }

    /// Remove the given id and its corresponding name from the map.
    pub fn remove_id(&self, id: &SourceId) {
        if let Some((_, name)) = self.id_to_name.remove(id) {
            self.name_to_id.remove(&name);
        } else {
            unreachable!()
        }
    }

    /// Remove the given name and its corresponding id from the map.
    ///
    /// # Panics
    ///
    /// Panics if the given name doesn't exist in the map.
    pub fn remove_name(&self, name: &String) {
        if let Some((_, id)) = self.name_to_id.remove(name) {
            self.id_to_name.remove(&id);
        } else {
            unreachable!()
        }
    }

    /// Rename the given filename without changing its id.
    pub fn rename_id(&self, id: &SourceId, new_name: String) {
        if let Some(old_name) = self.id_to_name.get(id) {
            assert!(self.name_to_id.remove(&*old_name).is_some());

            self.name_to_id.insert(new_name.clone(), *id);
            self.id_to_name.insert(*id, new_name);
        } else {
            // Ideally, this is the only structure which generates source ids,
            // which means that any id passed to this should be valid.
            unreachable!()
        };
    }

    /// Rename `old_name` to `new_name` without changing its id.
    ///
    /// # Panics
    ///
    /// Panics if the given name doesn't exist in the map.
    pub fn rename_name(&self, old_name: &String, new_name: String) {
        if let Some((_, id)) = self.name_to_id.remove(old_name) {
            assert!(self.id_to_name.remove(&id).is_some());

            self.name_to_id.insert(new_name.clone(), id);
            self.id_to_name.insert(id, new_name);
        } else {
            unreachable!()
        }
    }
}
