use std::collections::HashMap;

use crate::node::{AigAndGate, AigInput, Node};
use crate::reference::Ref;

/// And-Inverter Graph.
pub struct Aig {
    inputs: Vec<AigInput>,
    outputs: Vec<Ref>,
    and_gates: Vec<AigAndGate>,
    mapping: HashMap<u32, Node>,
}

impl Aig {
    pub fn new(
        inputs: Vec<AigInput>,
        outputs: Vec<Ref>,
        and_gates: Vec<AigAndGate>,
        mapping: HashMap<u32, Node>,
    ) -> Self {
        assert_eq!(inputs.len() + and_gates.len(), mapping.len());
        assert!(inputs.iter().all(|input| mapping.contains_key(&input.id)));
        assert!(outputs
            .iter()
            .all(|output| mapping.contains_key(&output.id())));
        assert!(and_gates.iter().all(|gate| mapping.contains_key(&gate.id)));
        Self {
            inputs,
            outputs,
            and_gates,
            mapping,
        }
    }
}

impl Aig {
    pub fn node(&self, id: u32) -> Node {
        self.mapping[&id]
    }

    pub fn is_input(&self, id: u32) -> bool {
        matches!(self.node(id), Node::Input(_))
    }
    pub fn is_and_gate(&self, id: u32) -> bool {
        matches!(self.node(id), Node::AndGate(_))
    }

    pub fn input(&self, id: u32) -> AigInput {
        match self.node(id) {
            Node::Input(input) => input,
            _ => panic!("Node with id {} is not an input", id),
        }
    }

    pub fn and_gate(&self, id: u32) -> AigAndGate {
        match self.node(id) {
            Node::AndGate(gate) => gate,
            _ => panic!("Node with id {} is not an AND gate", id),
        }
    }
}
