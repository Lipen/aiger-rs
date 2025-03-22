use std::fs::File;
use std::io::Read;
use std::path::Path;

use eyre::WrapErr;

use crate::aig::Aig;
use crate::aiger::{Literal, Reader, Record};
use crate::reference::Ref;

impl Aig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> eyre::Result<Self> {
        let path = path.as_ref();
        log::debug!("Reading AIG from {}", path.display());
        let file =
            File::open(path).wrap_err_with(|| format!("Failed to open {}", path.display()))?;
        Self::from_reader(file)
    }

    pub fn from_reader(reader: impl Read) -> eyre::Result<Self> {
        let reader = Reader::from_reader(reader)?;
        let mut aig = Aig::default();
        for record in reader.records() {
            let record = record?;
            match record {
                Record::Input { id: input } => {
                    assert!(!input.is_negated());
                    aig.add_input(input.index());
                }
                Record::Latch { .. } => {
                    todo!("latches are not supported yet")
                }
                Record::Output { id: output } => {
                    aig.add_output(lit2ref(output));
                }
                Record::AndGate { id: output, inputs } => {
                    assert!(!output.is_negated());
                    let args = [lit2ref(inputs[0]), lit2ref(inputs[1])];
                    aig.add_and_gate(output.index(), args);
                }
                Record::Symbol { .. } => {
                    // do nothing
                }
            }
        }
        Ok(aig)
    }
}

const fn lit2ref(lit: Literal) -> Ref {
    Ref::new(lit.index(), lit.is_negated())
}

#[cfg(test)]
mod tests {
    use crate::node::{AigAndGate, AigInput, Node};

    use super::*;

    fn parse_aig(input: &str) -> Aig {
        Aig::from_reader(input.as_bytes()).unwrap()
    }

    #[test]
    fn test_parse_aig() {
        #[rustfmt::skip]
        let aig = parse_aig(concat!(
        "aag 3 2 0 1 1\n",
        "2\n",
        "4\n",
        "6\n",
        "6 2 5\n",
        ));
        assert_eq!(aig.inputs(), &[1, 2]);
        assert_eq!(aig.outputs(), &[Ref::positive(3)]);
        assert_eq!(aig.node(1), Node::Input(AigInput { id: 1 }));
        assert_eq!(aig.node(2), Node::Input(AigInput { id: 2 }));
        assert_eq!(
            aig.gate(3),
            AigAndGate {
                id: 3,
                args: [Ref::positive(1), Ref::negative(2)]
            }
        );
    }
}
