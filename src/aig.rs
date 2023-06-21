use std::collections::HashMap;

use crate::node::{AigAndGate, AigInput, Node};
use crate::reference::Ref;
use crate::utils::{toposort_backward, toposort_forward};

/// And-Inverter Graph.
pub struct Aig {
    inputs: Vec<AigInput>,
    outputs: Vec<Ref>,
    gates: Vec<AigAndGate>,
    mapping: HashMap<u32, Node>,
}

impl Aig {
    pub fn new(inputs: Vec<AigInput>, outputs: Vec<Ref>, gates: Vec<AigAndGate>) -> Self {
        let mut mapping: HashMap<u32, Node> = HashMap::new();
        for &input in inputs.iter() {
            let old = mapping.insert(input.id, Node::Input(input));
            assert!(old.is_none(), "Duplicate input id {}", input.id);
        }
        for &gate in gates.iter() {
            let old = mapping.insert(gate.id, Node::AndGate(gate));
            assert!(old.is_none(), "Duplicate gate id {}", gate.id);
        }
        for output in outputs.iter() {
            assert!(
                mapping.contains_key(&output.id()),
                "Output id {} does not exist",
                output.id()
            );
        }
        Self {
            inputs,
            outputs,
            gates,
            mapping,
        }
    }
}

impl Aig {
    pub fn inputs(&self) -> &[AigInput] {
        &self.inputs
    }
    pub fn outputs(&self) -> &[Ref] {
        &self.outputs
    }
    pub fn gates(&self) -> &[AigAndGate] {
        &self.gates
    }
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.mapping.values()
    }

    pub fn node(&self, id: u32) -> Node {
        self.mapping[&id]
    }
    pub fn input(&self, id: u32) -> AigInput {
        match self.node(id) {
            Node::Input(input) => input,
            _ => panic!("Node with id {} is not an input", id),
        }
    }
    pub fn gate(&self, id: u32) -> AigAndGate {
        match self.node(id) {
            Node::AndGate(gate) => gate,
            _ => panic!("Node with id {} is not an AND gate", id),
        }
    }

    pub fn is_input(&self, id: u32) -> bool {
        matches!(self.node(id), Node::Input(_))
    }
    pub fn is_gate(&self, id: u32) -> bool {
        matches!(self.node(id), Node::AndGate(_))
    }
    pub fn contains(&self, id: u32) -> bool {
        self.mapping.contains_key(&id)
    }
}

impl Aig {
    pub fn layers_forward(&self) -> impl Iterator<Item = Vec<u32>> {
        toposort_forward(&self.dependency_graph()).map(|mut xs| {
            xs.sort();
            xs
        })
    }

    pub fn layers_backward(&self) -> impl Iterator<Item = Vec<u32>> {
        toposort_backward(&self.dependency_graph()).map(|mut xs| {
            xs.sort();
            xs
        })
    }

    fn dependency_graph(&self) -> HashMap<u32, Vec<u32>> {
        self.nodes()
            .map(|node| {
                (
                    node.id(),
                    node.children()
                        .into_iter()
                        .map(|c| c.id())
                        .collect::<Vec<_>>(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layers() {
        let x1 = AigInput { id: 1 };
        let x2 = AigInput { id: 2 };
        let x3 = AigInput { id: 3 };
        let g1 = AigAndGate {
            id: 4,
            args: [Ref::positive(x1.id), Ref::positive(x2.id)],
        };
        let g2 = AigAndGate {
            id: 5,
            args: [Ref::positive(g1.id), Ref::positive(x3.id)],
        };
        let g3 = AigAndGate {
            id: 6,
            args: [Ref::positive(x1.id), Ref::positive(g2.id)],
        };
        let inputs = vec![x1, x2, x3];
        let outputs = vec![Ref::positive(g3.id)];
        let gates = vec![g1, g2, g3];
        let aig = Aig::new(inputs, outputs, gates);

        let mut layers_forward = aig.layers_forward().collect::<Vec<_>>();
        assert_eq!(layers_forward.len(), 4);
        assert_eq!(layers_forward[0], vec![6]);
        assert_eq!(layers_forward[1], vec![5]);
        assert_eq!(layers_forward[2], vec![3, 4]);
        assert_eq!(layers_forward[3], vec![1, 2]);

        let mut layers_backward = aig.layers_backward().collect::<Vec<_>>();
        assert_eq!(layers_backward.len(), 4);
        assert_eq!(layers_backward[0], vec![1, 2, 3]);
        assert_eq!(layers_backward[1], vec![4]);
        assert_eq!(layers_backward[2], vec![5]);
        assert_eq!(layers_backward[3], vec![6]);
    }
}
