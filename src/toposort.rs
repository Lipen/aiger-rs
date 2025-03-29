use std::collections::HashMap;
use std::hash::Hash;

/// Performs a topological sorting of a directed graph.
/// Returns an iterator of layers.
/// The first layer consists of nodes that have no incoming edges (in-degree 0).
///
/// # Panics
///
/// Panics if the input graph contains a cycle.
pub fn toposort_layers<T, I>(graph: &HashMap<T, I>) -> impl Iterator<Item = Vec<T>> + '_
where
    T: Eq + Hash + Clone,
    for<'a> &'a I: IntoIterator<Item = &'a T>,
{
    // Compute in-degree for each node
    let mut in_degree = HashMap::new();
    for (node, neighbors) in graph {
        in_degree.entry(node.clone()).or_insert(0);
        for neighbor in neighbors {
            *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
        }
    }

    // Initialize with nodes that have no incoming edges
    let mut current_layer: Vec<T> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(node, _)| node.clone())
        .collect();

    let mut processed = 0;
    let total = in_degree.len();

    std::iter::from_fn(move || {
        // Check for completion or cycles
        if current_layer.is_empty() {
            if processed != total {
                panic!("Graph contains a cycle");
            }
            return None;
        }

        // Yield current layer and prepare next
        let layer = std::mem::take(&mut current_layer);
        processed += layer.len();

        let mut next_layer: Vec<T> = Vec::new();
        for node in &layer {
            if let Some(neighbors) = graph.get(node) {
                for neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        next_layer.push(neighbor.clone());
                    }
                }
            }
        }

        current_layer = next_layer;
        Some(layer)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sort_layers<T: Ord>(layers: &mut [Vec<T>]) {
        for layer in layers.iter_mut() {
            layer.sort();
        }
    }

    #[test]
    fn test_example() {
        let mut graph = HashMap::new();
        graph.insert(3, vec![10, 8]);
        graph.insert(5, vec![11]);
        graph.insert(7, vec![8, 11]);
        graph.insert(8, vec![9]);
        graph.insert(11, vec![9, 2, 10]);

        let mut layers = toposort_layers(&graph);
        sort_layers(&mut layers);
        assert_eq!(layers, vec![vec![3, 5, 7], vec![8, 11], vec![2, 9, 10]]);
    }

    #[test]
    fn test_linear_graph() {
        let mut graph = HashMap::new();
        graph.insert("A", vec!["B"]);
        graph.insert("B", vec!["C"]);
        graph.insert("C", vec!["D"]);

        let mut layers = toposort_layers(&graph);
        sort_layers(&mut layers);
        assert_eq!(layers, vec![vec!["A"], vec!["B"], vec!["C"], vec!["D"]]);
    }

    #[test]
    fn test_multiple_nodes_in_layer() {
        let mut graph = HashMap::new();
        graph.insert("A", vec!["B", "C"]);
        graph.insert("B", vec!["D"]);
        graph.insert("C", vec!["D"]);

        let mut layers = toposort_layers(&graph);
        sort_layers(&mut layers);
        assert_eq!(layers, vec![vec!["A"], vec!["B", "C"], vec!["D"]]);
    }

    #[test]
    fn test_complex_dag() {
        let mut graph = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![4]);
        graph.insert(5, vec![6]);

        let mut layers = toposort_layers(&graph);
        sort_layers(&mut layers);
        assert_eq!(layers, vec![vec![1, 5], vec![2, 3, 6], vec![4]]);
    }

    #[test]
    fn test_single_node() {
        let graph = HashMap::from([(1, vec![])]);
        let layers = toposort_layers(&graph);
        assert_eq!(layers, vec![vec![1]]);
    }

    #[test]
    fn test_node_not_in_keys() {
        let mut graph = HashMap::new();
        graph.insert("A", vec!["B"]);

        let mut layers = toposort_layers(&graph);
        sort_layers(&mut layers);
        assert_eq!(layers, vec![vec!["A"], vec!["B"]]);
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<i32, Vec<i32>> = HashMap::new();
        let layers = toposort_layers(&graph);
        assert!(layers.is_empty());
    }

    #[test]
    #[should_panic]
    fn test_cyclic_graph() {
        let mut graph = HashMap::new();
        graph.insert("A", vec!["B"]);
        graph.insert("B", vec!["C"]);
        graph.insert("C", vec!["A"]);

        let layers = toposort_layers(&graph);
        layers.for_each(drop);
    }

    #[test]
    fn test_disconnected_components() {
        let mut graph = HashMap::new();
        graph.insert("A", vec!["B"]);
        graph.insert("C", vec!["D"]);

        let mut layers = toposort_layers(&graph);
        sort_layers(&mut layers);
        assert_eq!(layers, vec![vec!["A", "C"], vec!["B", "D"]]);
    }
}
