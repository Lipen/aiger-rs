use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};

use crate::node::{AigAndGate, AigInput, Node};
use crate::reference::Ref;
use crate::toposort::{toposort_backward, toposort_forward};

/// And-Inverter Graph.
pub struct Aig {
    nodes: HashMap<u32, Node>,
    inputs: Vec<u32>,
    outputs: Vec<Ref>,
}

impl Aig {
    pub const fn new(nodes: HashMap<u32, Node>, inputs: Vec<u32>, outputs: Vec<Ref>) -> Self {
        Self {
            nodes,
            inputs,
            outputs,
        }
    }
}

impl Default for Aig {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

impl Display for Aig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Aig(inputs: {}, outputs: {})",
            self.inputs.len(),
            self.outputs.len(),
        )
    }
}

impl Aig {
    pub fn inputs(&self) -> &[u32] {
        &self.inputs
    }
    pub fn outputs(&self) -> &[Ref] {
        &self.outputs
    }
    pub fn nodes(&self) -> &HashMap<u32, Node> {
        &self.nodes
    }

    pub fn is_constant(&self, id: u32) -> bool {
        id == 0 || id == 1
    }
    pub fn is_input(&self, id: u32) -> bool {
        if self.is_constant(id) {
            return false;
        }
        matches!(self.nodes[&id], Node::Input(..))
    }
    pub fn is_gate(&self, id: u32) -> bool {
        if self.is_constant(id) {
            return false;
        }
        matches!(self.nodes[&id], Node::AndGate(..))
    }

    pub fn contains(&self, id: u32) -> bool {
        if self.is_constant(id) {
            return true;
        }
        self.nodes.contains_key(&id)
    }

    pub fn node(&self, id: u32) -> Node {
        match id {
            0 => Node::constant(false),
            1 => Node::constant(true),
            _ => self.nodes[&id],
        }
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

    pub fn add_input(&mut self, id: u32) {
        assert!(!self.nodes.contains_key(&id));
        assert!(!self.inputs.contains(&id));
        self.nodes.insert(id, Node::input(id));
        self.inputs.push(id);
    }

    pub fn add_output(&mut self, output: Ref) {
        self.outputs.push(output);
    }

    pub fn add_and_gate(&mut self, id: u32, args: [Ref; 2]) {
        assert!(!self.nodes.contains_key(&id));
        for arg in args.iter() {
            assert!(self.nodes.contains_key(&arg.id()));
        }
        self.nodes.insert(id, Node::and_gate(id, args));
    }
}

// Layers
impl Aig {
    /// Return the iterator of 'backward' layers in the AIG.
    /// The first 'backward' layer consists of all inputs.
    pub fn layers_input(&self) -> impl Iterator<Item = Vec<u32>> {
        toposort_backward(&self.dependency_graph()).map(|mut xs| {
            xs.sort();
            xs
        })
    }

    /// Return the iterator of 'forward' layers in the AIG.
    /// The first 'forward' layer consists of all outputs.
    pub fn layers_output(&self) -> impl Iterator<Item = Vec<u32>> {
        toposort_forward(&self.dependency_graph()).map(|mut xs| {
            xs.sort();
            xs
        })
    }

    fn dependency_graph(&self) -> HashMap<u32, Vec<u32>> {
        self.nodes()
            .iter()
            .map(|(&id, node)| {
                (
                    id,
                    node.children().iter().map(|c| c.id()).collect::<Vec<_>>(),
                )
            })
            .collect()
    }
}

// Evaluation
impl Aig {
    pub fn eval(&self, input_values: Vec<bool>) -> BTreeMap<u32, bool> {
        assert_eq!(input_values.len(), self.inputs.len());

        let mut values = BTreeMap::new();

        for layer in self.layers_input() {
            for id in layer {
                match self.node(id) {
                    Node::Constant(value) => {
                        values.insert(id, value);
                    }
                    Node::Input(input) => {
                        let i = self.inputs.iter().position(|&x| x == input.id).unwrap();
                        let value = input_values[i];
                        values.insert(id, value);
                    }
                    Node::AndGate(gate) => {
                        let left = values[&gate.args[0].id()] ^ gate.args[0].is_negated();
                        let right = values[&gate.args[1].id()] ^ gate.args[1].is_negated();
                        let value = left && right;
                        values.insert(id, value);
                    }
                }
            }
        }

        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layers() {
        let mut aig = Aig::default();

        aig.add_input(1);
        aig.add_input(2);
        aig.add_input(3);
        aig.add_and_gate(4, [Ref::positive(1), Ref::positive(2)]);
        aig.add_and_gate(5, [Ref::positive(4), Ref::positive(3)]);
        aig.add_and_gate(6, [Ref::positive(1), Ref::positive(5)]);
        aig.add_output(Ref::positive(6));

        let layers_input = aig.layers_input().collect::<Vec<_>>();
        assert_eq!(layers_input.len(), 4);
        assert_eq!(layers_input[0], vec![1, 2, 3]);
        assert_eq!(layers_input[1], vec![4]);
        assert_eq!(layers_input[2], vec![5]);
        assert_eq!(layers_input[3], vec![6]);

        let layers_output = aig.layers_output().collect::<Vec<_>>();
        assert_eq!(layers_output.len(), 4);
        assert_eq!(layers_output[0], vec![6]);
        assert_eq!(layers_output[1], vec![5]);
        assert_eq!(layers_output[2], vec![3, 4]);
        assert_eq!(layers_output[3], vec![1, 2]);
    }

    #[test]
    fn test_eval() {
        let mut aig = Aig::default();

        aig.add_input(1);
        aig.add_input(2);
        aig.add_input(3);

        // g1 = x1 and x2
        aig.add_and_gate(4, [Ref::positive(1), Ref::positive(2)]);
        // g2 = ~g1 and x3
        aig.add_and_gate(5, [Ref::negative(4), Ref::positive(3)]);
        // g3 = x1 and ~g2
        aig.add_and_gate(6, [Ref::positive(1), Ref::negative(5)]);

        aig.add_output(Ref::positive(6));

        let input_values = vec![true, false, true]; // [x1, x2, x3]
        println!("input: {:?}", input_values);
        let values = aig.eval(input_values);
        println!("values: {:?}", values);
        assert_eq!(values[&1], true); // x1
        assert_eq!(values[&2], false); // x2
        assert_eq!(values[&3], true); // x3
        assert_eq!(values[&4], false); // g1 = x1 and x2
        assert_eq!(values[&5], true); // g2 = ~g1 and x3
        assert_eq!(values[&6], false); // g3 = x1 and ~g2
    }
}
