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
    pub fn children(&self) -> &[Ref] {
        match self {
            Node::Input { .. } => &[],
            Node::AndGate(AigAndGate { args, .. }) => args,
            // Node::True => &[],
            // Node::False => &[],
        }
    }
}
