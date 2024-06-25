use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Performs a "forward" topological sort on a directed graph.
/// Returns an iterator over layers.
/// Each layer is [`Vec<T>`].
/// The first layer of forward toposort consists of nodes that have no incoming edges (in-degree 0).
pub fn toposort_forward<T>(graph: &HashMap<T, Vec<T>>) -> ForwardTopoSort<T>
where
    T: Hash + Eq + Clone,
{
    ForwardTopoSort::new(graph)
}

pub struct ForwardTopoSort<T> {
    graph: HashMap<T, Vec<T>>,
    in_degree: HashMap<T, usize>,
    queue: VecDeque<T>,
    circular_dependency: bool,
}

impl<T> ForwardTopoSort<T>
where
    T: Hash + Eq + Clone,
{
    pub fn new(graph: &HashMap<T, Vec<T>>) -> Self {
        // Count in-degree for each node.
        let mut in_degree: HashMap<T, usize> = HashMap::new();
        for node in graph.values().flat_map(|v| v.iter()) {
            let count = in_degree.entry(node.clone()).or_default();
            *count += 1;
        }

        // Create a queue and initialize it with nodes that have no incoming edges.
        // These are potential starting points for the topological sort.
        let queue: VecDeque<_> = graph
            .keys()
            .filter(|node| !in_degree.contains_key(node))
            .cloned()
            .collect();

        Self {
            graph: graph.clone(),
            in_degree,
            queue,
            circular_dependency: false,
        }
    }
}

impl<T> Iterator for ForwardTopoSort<T>
where
    T: Hash + Eq + Clone,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            if !self.circular_dependency {
                for &count in self.in_degree.values() {
                    if count > 0 {
                        self.circular_dependency = true;
                        panic!("Circular dependency detected");
                    }
                }
            }
            return None;
        }

        let mut layer = Vec::new();
        for _ in 0..self.queue.len() {
            let node = self.queue.pop_front().unwrap();

            if let Some(edges) = self.graph.remove(&node) {
                for edge in edges {
                    let count = self.in_degree.get_mut(&edge).unwrap();
                    *count -= 1;

                    if *count == 0 {
                        self.queue.push_back(edge);
                    }
                }
            }
            layer.push(node);
        }
        Some(layer)
    }
}

/// Performs a "backward" topological sort on a directed graph.
/// Returns an iterator over layers.
/// Each layer is [`Vec<T>`].
/// The first layer of backward toposort consists of nodes that have no outgoing edges.
pub fn toposort_backward<T>(graph: &HashMap<T, Vec<T>>) -> BackwardTopoSort<T>
where
    T: Hash + Eq + Clone,
{
    BackwardTopoSort::new(graph)
}

pub struct BackwardTopoSort<T> {
    data: HashMap<T, HashSet<T>>,
}

impl<T> BackwardTopoSort<T>
where
    T: Hash + Eq + Clone,
{
    pub fn new(graph: &HashMap<T, Vec<T>>) -> Self {
        // Local mutable data:
        let mut data: HashMap<T, HashSet<T>> = HashMap::with_capacity(graph.len());

        // Add all deps to the map:
        for (item, deps) in graph.iter() {
            data.insert(item.clone(), deps.iter().cloned().collect());
        }

        // Find all items without deps and add them explicitly to the map:
        for item in graph.values().flat_map(|v| v.iter()) {
            data.entry(item.clone()).or_default();
        }

        BackwardTopoSort { data }
    }
}

impl<T> Iterator for BackwardTopoSort<T>
where
    T: Hash + Eq + Clone,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // New layer is a list of items without dependencies:
        let layer: Vec<T> = self
            .data
            .iter()
            .filter_map(|(item, deps)| {
                if deps.is_empty() {
                    Some(item.clone())
                } else {
                    None
                }
            })
            .collect();

        // New layer can be empty in two cases:
        //  (1) `data` is empty (this is OK, no more layers),
        //  (2) or there is a circular dependency in `data`.
        if layer.is_empty() {
            // If `data` is not empty, we have a cycle:
            // (note: `data` contains the cycle itself)
            assert!(self.data.is_empty(), "Circular dependency detected");

            return None;
        }

        // Remove keys without deps (new layer):
        for item in &layer {
            self.data.remove(item);
        }

        // Reduce deps:
        for deps in self.data.values_mut() {
            for item in &layer {
                deps.remove(item);
            }
        }

        // Return non-empty layer:
        Some(layer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_toposort() {
        let mut graph = HashMap::new();
        graph.insert("g1", vec!["x1", "x2"]);
        graph.insert("g2", vec!["g1", "x3"]);
        graph.insert("g3", vec!["x1", "g2"]);
        let layers = toposort_forward(&graph).collect::<Vec<_>>();
        assert_eq!(layers[0], vec!["g3"]);
        assert_eq!(layers[1], vec!["g2"]);
        assert_eq!(layers[2], vec!["g1", "x3"]);
        assert_eq!(layers[3], vec!["x1", "x2"]);
    }

    #[test]
    #[should_panic(expected = "Circular dependency detected")]
    fn test_forward_toposort_cycle() {
        let mut graph = HashMap::new();
        graph.insert("x1", vec!["x2"]);
        graph.insert("x2", vec!["x3"]);
        graph.insert("x3", vec!["x1"]);
        toposort_forward(&graph).for_each(|_| {});
    }

    #[test]
    fn test_backward_toposort() {
        let mut graph = HashMap::new();
        graph.insert("g1", vec!["x1", "x2"]);
        graph.insert("g2", vec!["g1", "x3"]);
        graph.insert("g3", vec!["x1", "g2"]);
        let mut layers = toposort_backward(&graph);
        assert_eq!(
            layers.next().unwrap().into_iter().collect::<HashSet<_>>(),
            HashSet::from(["x1", "x2", "x3"])
        );
        assert_eq!(
            layers.next().unwrap().into_iter().collect::<HashSet<_>>(),
            HashSet::from(["g1"])
        );
        assert_eq!(
            layers.next().unwrap().into_iter().collect::<HashSet<_>>(),
            HashSet::from(["g2"])
        );
        assert_eq!(
            layers.next().unwrap().into_iter().collect::<HashSet<_>>(),
            HashSet::from(["g3"])
        );
    }

    #[test]
    #[should_panic(expected = "Circular dependency detected")]
    fn test_backward_toposort_cycle() {
        let mut graph = HashMap::new();
        graph.insert("x1", vec!["x2"]);
        graph.insert("x2", vec!["x3"]);
        graph.insert("x3", vec!["x1"]);
        toposort_backward(&graph).for_each(|_| {});
    }
}
