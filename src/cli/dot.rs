use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use zippy_frontend::checked::ItemIndex;
use zippy_frontend::dependencies::Dependencies;

use crate::pretty::Prettier;

pub struct GraphViz {
    edges: Vec<(ItemIndex, ItemIndex)>,
    ids: HashMap<ItemIndex, usize>,
    names: HashMap<ItemIndex, String>,
}

impl GraphViz {
    pub fn new(prettier: &Prettier, deps: Dependencies) -> Self {
        let mut id = 0;
        let mut edges = Vec::with_capacity(deps.graph.len());
        let mut ids = HashMap::new();

        for (from, outgoing) in deps.graph {
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

        let mut names = HashMap::new();
        for (index, item_names) in deps.names {
            let result = item_names
                .into_iter()
                .map(|name| prettier.pretty_item_name(name))
                .join(", ");

            let result = if result.is_empty() {
                let id = match ids.entry(index) {
                    Entry::Vacant(e) => {
                        let i = id;
                        id += 1;
                        e.insert(i);
                        i
                    }

                    Entry::Occupied(e) => *e.get(),
                };

                format!("item#{id}")
            } else {
                result
            };

            names.insert(index, result);
        }

        Self { edges, ids, names }
    }

    pub fn render<W: std::io::Write>(&self, output: &mut W) -> dot2::Result {
        dot2::render(self, output)
    }
}

impl<'a> dot2::Labeller<'a> for GraphViz {
    type Node = ItemIndex;
    type Edge = (ItemIndex, ItemIndex);
    type Subgraph = ();

    fn graph_id(&'a self) -> dot2::Result<dot2::Id<'a>> {
        dot2::Id::new("dependencies")
    }

    fn node_id(&'a self, n: &Self::Node) -> dot2::Result<dot2::Id<'a>> {
        dot2::Id::new(format!("N{}", self.ids.get(n).unwrap()))
    }

    fn node_label(&'a self, n: &Self::Node) -> dot2::Result<dot2::label::Text<'a>> {
        let name = self
            .names
            .get(n)
            .map(|s| Cow::Borrowed(s.as_str()))
            .unwrap_or_else(|| {
                let id = *self.ids.get(n).expect("all names have an id");
                Cow::Owned(format!("item#{id}"))
            });

        Ok(dot2::label::Text::LabelStr(name))
    }
}

impl<'a> dot2::GraphWalk<'a> for GraphViz {
    type Node = ItemIndex;
    type Edge = (ItemIndex, ItemIndex);
    type Subgraph = ();

    fn nodes(&'a self) -> dot2::Nodes<'a, Self::Node> {
        let mut nodes = HashSet::new();
        for (from, to) in self.edges.iter() {
            nodes.insert(*from);
            nodes.insert(*to);
        }

        nodes.into_iter().collect()
    }

    fn edges(&'a self) -> dot2::Edges<'a, Self::Edge> {
        (&self.edges[..]).into()
    }

    fn source(&'a self, edge: &Self::Edge) -> Self::Node {
        edge.0
    }

    fn target(&'a self, edge: &Self::Edge) -> Self::Node {
        edge.1
    }
}
