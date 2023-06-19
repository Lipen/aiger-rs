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
    pub(crate) fn new(
        inputs: Vec<AigInput>,
        outputs: Vec<Ref>,
        and_gates: Vec<AigAndGate>,
        mapping: HashMap<u32, Node>,
    ) -> Self {
        Self {
            inputs,
            outputs,
            and_gates,
            mapping,
        }
    }
}
