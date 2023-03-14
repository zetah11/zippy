use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use zippy_frontend::dependencies::ItemOrAlias;

use crate::database::Database;
use crate::pretty::Prettier;

pub struct GraphViz<'db> {
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
