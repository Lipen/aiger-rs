use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use eyre::WrapErr;

use crate::aig::Aig;
use crate::aiger::Header;

impl Aig {
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> eyre::Result<()> {
        let path = path.as_ref();
        log::debug!("Writing AIG to {}", path.display());
        let f =
            File::create(path).wrap_err_with(|| format!("Failed to create {}", path.display()))?;
        let mut f = BufWriter::new(f);
        self.write(&mut f)
    }

    pub fn write_to_string(&self) -> eyre::Result<String> {
        log::debug!("Writing AIG to string");
        let mut buf = Vec::new();
        self.write(&mut buf)?;
        let s = String::from_utf8(buf)?;
        Ok(s)
    }

    pub fn write(&self, writer: &mut impl Write) -> eyre::Result<()> {
        let header = Header {
            m: *self.nodes().keys().max().unwrap() as usize,
            i: self.inputs().len(),
            l: 0,
            o: self.outputs().len(),
            a: self.nodes().len() - self.inputs().len(),
        };
        // Header:
        writeln!(writer, "{}", header)?;
        // Inputs:
        for &input in self.inputs() {
            writeln!(writer, "{}", input * 2)?;
        }
        // Outputs:
        for output in self.outputs() {
            writeln!(writer, "{}", output.raw())?;
        }
        // Gates:
        let mut gates: Vec<u32> = self.and_gates().map(|g| g.id).collect();
        gates.sort();
        for id in gates {
            let gate = self.gate(id);
            let [left, right] = gate.args;
            writeln!(writer, "{} {} {}", id * 2, left.raw(), right.raw())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use crate::reference::Ref;

    #[test]
    fn test_write_aiger() {
        let mut aig = Aig::default();
        aig.add_input(1);
        aig.add_input(2);
        aig.add_and_gate(3, [Ref::negative(1), Ref::positive(2)]);
        aig.add_and_gate(4, [Ref::negative(3), Ref::FALSE]);
        aig.add_output(Ref::negative(3));
        aig.add_output(Ref::positive(4));
        let s = aig.write_to_string().unwrap();
        let expected = indoc! {"
            aag 4 2 0 2 2
            2
            4
            7
            8
            6 3 4
            8 7 0
        "};
        assert_eq!(s, expected);
    }
}
