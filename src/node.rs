use crate::reference::Ref;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Node {
    Zero,
    Input(AigInput),
    Latch(AigLatch),
    AndGate(AigAndGate),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AigInput {
    pub id: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AigLatch {
    pub id: u32,
    pub next: Ref,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AigAndGate {
    pub id: u32,
    pub args: [Ref; 2],
}

impl Node {
    pub const ZERO: Self = Node::Zero;

    pub const fn input(id: u32) -> Self {
        Node::Input(AigInput { id })
    }
    pub const fn latch(id: u32, next: Ref) -> Self {
        Node::Latch(AigLatch { id, next })
    }
    pub const fn and_gate(id: u32, args: [Ref; 2]) -> Self {
        Node::AndGate(AigAndGate { id, args })
    }

    pub const fn id(&self) -> u32 {
        match self {
            Node::Zero => 0,
            Node::Input(input) => input.id,
            Node::Latch(latch) => latch.id,
            Node::AndGate(gate) => gate.id,
        }
    }

    pub const fn children(&self) -> &[Ref] {
        match self {
            Node::Zero => &[],
            Node::Input(_) => &[],
            Node::Latch(_) => &[],
            Node::AndGate(gate) => &gate.args,
        }
    }
}
