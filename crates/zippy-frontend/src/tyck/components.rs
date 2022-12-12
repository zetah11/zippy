//! Identifies the strongly connected components in the program dependency
//! cycle.
//!
//! This implementation is based on Tarjan's algorithm, which also produces
//! a reverse topological sort. This means that the typer can go through the
//! components in the order that they are produced.

use std::collections::{HashMap, HashSet};

use zippy_common::thir::Decls;

use super::dependency::{DefIndex, Dependencies};

#[derive(Debug)]
pub struct Components {
    index: usize,

    indicies: HashMap<DefIndex, usize>,
    lowlinks: HashMap<DefIndex, usize>,

    stack: Vec<DefIndex>,
    on_stack: HashSet<DefIndex>,

    components: Vec<HashSet<DefIndex>>,
}

impl Components {
    pub fn find(decls: &Decls) -> Vec<HashSet<DefIndex>> {
        let graph = Dependencies::find(decls);
        let mut finder = Self {
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

        finder.components
    }

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
