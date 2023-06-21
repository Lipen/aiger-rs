use crate::reference::Ref;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Node {
    Input(AigInput),
    AndGate(AigAndGate),
    // True,
    // False,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AigInput {
    pub id: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AigAndGate {
    pub id: u32,
    pub args: [Ref; 2],
}

impl Node {
    pub const fn id(&self) -> u32 {
        match self {
            Node::Input(input) => input.id,
            Node::AndGate(gate) => gate.id,
        }
    }

    pub const fn children(&self) -> &[Ref] {
        match self {
            Node::Input(_) => &[],
            Node::AndGate(gate) => &gate.args,
        }
    }
}
