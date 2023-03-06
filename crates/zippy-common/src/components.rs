//! Implements Tarjan's algorithm, which produces a topological ordering of
//! every strongly connected component in a graph. This is used generically by
//! other parts of the compiler.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Compute a topologically sorted list of every strongly connected component in
/// the given graph. The graph is provided as a mapping from nodes to their
/// outgoing edges.
pub fn find<T>(graph: &HashMap<T, HashSet<T>>) -> Vec<HashSet<T>>
where
    T: Copy + Eq + Hash,
{
    let mut finder = ComponentFinder::new();
    for vertex in graph.keys() {
        finder.connect(graph, *vertex);
    }

    finder.components
}

struct ComponentFinder<T> {
    index: usize,

    indicies: HashMap<T, usize>,
    lowlinks: HashMap<T, usize>,

    stack: Vec<T>,
    on_stack: HashSet<T>,

    visited: HashSet<T>,
    components: Vec<HashSet<T>>,
}

impl<T> ComponentFinder<T>
where
    T: Copy + Eq + Hash,
{
    fn new() -> Self {
        Self {
            index: 0,
            indicies: HashMap::new(),
            lowlinks: HashMap::new(),
            stack: Vec::new(),
            on_stack: HashSet::new(),

            visited: HashSet::new(),
            components: Vec::new(),
        }
    }

    fn connect(&mut self, graph: &HashMap<T, HashSet<T>>, vertex: T) {
        if !self.visited.insert(vertex) {
            return;
        }

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
                self.on_stack.remove(&child);
                component.insert(child);

                if child == vertex {
                    break;
                }
            }

            self.components.push(component);
        }
    }
}
