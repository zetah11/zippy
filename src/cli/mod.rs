mod diagnostic;
mod format;

#[cfg(test)]
mod tests;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use zippy_common::messages::Messages;
use zippy_common::source::Project;
use zippy_frontend::dependencies::{get_dependencies, ItemOrAlias};

use crate::database::Database;
use crate::pretty::Prettier;
use crate::project;
use crate::project::{source_name_from_path, FsProject, DEFAULT_ROOT_NAME};

use self::diagnostic::print_diagnostic;

/// Perform checks on the project.
pub fn check() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_module_level("salsa_2022", LevelFilter::Warn)
        .init()
        .unwrap();

    let cwd = std::env::current_dir()?;
    let mut database = Database::new();

    let project_name = cwd
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| DEFAULT_ROOT_NAME.to_string());
    let project = Project::new(&database, project_name);
    let project = FsProject::new(project).with_root(&cwd);

    let sources = project::get_project_sources(&cwd)
        .into_iter()
        .filter_map(|path| {
            let content = fs::read_to_string(&path).ok()?;
            let name = source_name_from_path(&database, Some(&project), &path);
            Some((path, name, content))
        })
        .collect();

    database.init_sources(sources);

    let mut messages = Vec::new();
    let prettier = Prettier::new(&database).with_full_name(true);
    let mut all_deps = HashMap::new();

    for module in database.get_modules() {
        let dependencies = get_dependencies(&database, module);
        messages.extend(get_dependencies::accumulated::<Messages>(&database, module));

        for (name, depends) in dependencies.dependencies(&database) {
            assert!(all_deps.insert(*name, depends.clone()).is_none());
        }
    }

    for message in messages {
        print_diagnostic(&database, Some(&project), &prettier, message)?;
    }

    for component in zippy_common::components::find(&all_deps) {
        for name in component {
            print!(
                "{}, ",
                match name {
                    ItemOrAlias::Item(name) => prettier.pretty_item_name(name),
                    ItemOrAlias::Alias(alias) => format!("<import {}>", alias.name.text(&database)),
                }
            );
        }

        println!()
    }

    let graph = std::fs::File::create("dependencies.dot")?;
    let mut writer = std::io::BufWriter::new(graph);

    GraphViz::new(&database, &prettier, all_deps).render(&mut writer)?;

    Ok(())
}

struct GraphViz<'db> {
    db: &'db Database,
    prettier: &'db Prettier<'db>,
    edges: Vec<(ItemOrAlias, ItemOrAlias)>,
    ids: HashMap<ItemOrAlias, usize>,
}

impl<'db> GraphViz<'db> {
    pub fn new(
        db: &'db Database,
        prettier: &'db Prettier,
        graph: HashMap<ItemOrAlias, HashSet<ItemOrAlias>>,
    ) -> Self {
        let mut id = 0;
        let mut edges = Vec::with_capacity(graph.len());
        let mut ids = HashMap::new();

        for (from, outgoing) in graph {
            if let Entry::Vacant(e) = ids.entry(from) {
                e.insert(id);
                id += 1;
            }

            for to in outgoing {
                if let Entry::Vacant(e) = ids.entry(to) {
                    e.insert(id);
                    id += 1;
                }

                edges.push((from, to));
            }
        }

        Self {
            db,
            prettier,
            edges,
            ids,
        }
    }

    pub fn render<W: std::io::Write>(&self, output: &mut W) -> dot2::Result {
        dot2::render(self, output)
    }
}

impl<'db> dot2::Labeller<'db> for GraphViz<'db> {
    type Node = ItemOrAlias;
    type Edge = (ItemOrAlias, ItemOrAlias);
    type Subgraph = ();

    fn graph_id(&'db self) -> dot2::Result<dot2::Id<'db>> {
        dot2::Id::new("dependencies")
    }

    fn node_id(&'db self, n: &Self::Node) -> dot2::Result<dot2::Id<'db>> {
        dot2::Id::new(format!("N{}", self.ids.get(n).unwrap()))
    }

    fn node_label(&'db self, n: &Self::Node) -> dot2::Result<dot2::label::Text<'db>> {
        Ok(dot2::label::Text::LabelStr(match n {
            ItemOrAlias::Item(name) => self.prettier.pretty_item_name(*name).into(),
            ItemOrAlias::Alias(alias) => format!("<import {}>", alias.name.text(self.db)).into(),
        }))
    }
}

impl<'db> dot2::GraphWalk<'db> for GraphViz<'db> {
    type Node = ItemOrAlias;
    type Edge = (ItemOrAlias, ItemOrAlias);
    type Subgraph = ();

    fn nodes(&'db self) -> dot2::Nodes<'db, Self::Node> {
        let mut nodes = HashSet::new();
        for (from, to) in self.edges.iter() {
            nodes.insert(*from);
            nodes.insert(*to);
        }

        nodes.into_iter().collect()
    }

    fn edges(&'db self) -> dot2::Edges<'db, Self::Edge> {
        (&self.edges[..]).into()
    }

    fn source(&'db self, edge: &Self::Edge) -> Self::Node {
        edge.0
    }

    fn target(&'db self, edge: &Self::Edge) -> Self::Node {
        edge.1
    }
}
