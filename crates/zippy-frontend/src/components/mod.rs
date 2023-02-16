//! This module is responsible for identifying the strongly connected components
//! in the program dependency graph. A strongly connected component is a set of
//! nodes where each node can be "reached" from every other -- essentially, this
//! captures all of the dependency cycles in every name definition.
//!
//! This implementation is based on Tarjan's algorithm, which also produces
//! a reverse topological sort. This means that the typer can go through the
//! components in the order that they are produced.

mod dependency;

pub use dependency::DefIndex;

use std::collections::{HashMap, HashSet};

use self::dependency::Dependencies;
use crate::resolved::Decls;
use crate::Db;

#[salsa::tracked]
pub struct Components {
    #[return_ref]
    pub ordered: Vec<HashSet<DefIndex>>,
}

/// Find all the strongly connected components for the given `Decls`.
#[salsa::tracked]
pub fn components(db: &dyn Db, decls: Decls) -> Components {
    let graph = Dependencies::find(db, decls);
    let mut finder = Finder {
        index: 0,
        indicies: HashMap::new(),
        lowlinks: HashMap::new(),
        stack: Vec::new(),
        on_stack: HashSet::new(),
        components: Vec::new(),
    };

    for name in graph.keys() {
        if !finder.indicies.contains_key(name) {
            finder.connect(&graph, *name);
        }
    }

    Components::new(db, finder.components)
}

#[derive(Debug)]
struct Finder {
    index: usize,

    indicies: HashMap<DefIndex, usize>,
    lowlinks: HashMap<DefIndex, usize>,

    stack: Vec<DefIndex>,
    on_stack: HashSet<DefIndex>,

    components: Vec<HashSet<DefIndex>>,
}

impl Finder {
    fn connect(&mut self, graph: &HashMap<DefIndex, HashSet<DefIndex>>, vertex: DefIndex) {
        self.indicies.insert(vertex, self.index);
        self.lowlinks.insert(vertex, self.index);
        self.index += 1;

        self.stack.push(vertex);
        self.on_stack.insert(vertex);

        for child in graph.get(&vertex).into_iter().flatten() {
            if !self.indicies.contains_key(child) {
                self.connect(graph, *child);
                let lowlink = *self
                    .lowlinks
                    .get(&vertex)
                    .unwrap()
                    .min(self.lowlinks.get(child).unwrap());
                self.lowlinks.insert(vertex, lowlink);
            } else if self.on_stack.contains(child) {
                let lowlink = *self
                    .lowlinks
                    .get(&vertex)
                    .unwrap()
                    .min(self.indicies.get(child).unwrap());
                self.lowlinks.insert(vertex, lowlink);
            }
        }

        if self.lowlinks.get(&vertex) == self.indicies.get(&vertex) {
            let mut component = HashSet::new();
            while let Some(child) = self.stack.pop() {
                component.insert(child);
                self.on_stack.remove(&child);

                if child == vertex {
                    break;
                }
            }

            self.components.push(component);
        }
    }
}
