use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use eyre::{ensure, eyre};
use nom::bytes::complete::tag;
use nom::character::complete::{space1, u32 as u32_parser};
use nom::sequence::preceded;
use nom::IResult;

use crate::aig::Aig;
use crate::node::{AigAndGate, AigInput};
use crate::reference::Ref;

impl Aig {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> eyre::Result<Self> {
        let file = File::open(path)?;
        Self::parse_lines(BufReader::new(file).lines().map(|r| r.unwrap()))
    }

    pub fn parse_str(s: &str) -> eyre::Result<Self> {
        Self::parse_lines(s.lines().map(|s| s.to_owned()))
    }

    pub fn parse_lines(mut lines: impl Iterator<Item = String>) -> eyre::Result<Self> {
        let header = parse_header(&lines.next().ok_or_else(|| eyre!("Missing header"))?)?;

        let mut inputs = Vec::with_capacity(header.inputs as usize);
        for _ in 0..header.inputs {
            let input = parse_input(&lines.next().ok_or_else(|| eyre!("Missing input"))?)?;
            ensure!(
                input.id <= header.max,
                "Input id {} is greater than max {}",
                input.id,
                header.max
            );
            inputs.push(input);
        }

        // TODO: parse latches
        ensure!(header.latches == 0, "Latches are not supported");

        let mut outputs = Vec::with_capacity(header.outputs as usize);
        for _ in 0..header.outputs {
            let output = parse_output(&lines.next().ok_or_else(|| eyre!("Missing output"))?)?;
            ensure!(
                output.id() <= header.max,
                "Output id {} is greater than max {}",
                output.id(),
                header.max
            );
            outputs.push(output);
        }

        let mut gates = Vec::with_capacity(header.gates as usize);
        for _ in 0..header.gates {
            let and = parse_and_gate(&lines.next().ok_or_else(|| eyre!("Missing gate"))?)?;
            ensure!(
                and.id <= header.max,
                "Gate id {} is greater than max {}",
                and.id,
                header.max
            );
            gates.push(and);
        }

        let aig = Aig::new(inputs, outputs, gates);
        Ok(aig)
    }
}

/// AIGER header: `'aag M I L O A'`, where `M >= I + L + A`.
struct Header {
    /// Maximum variable index.
    max: u32,
    /// Number of inputs.
    inputs: u32,
    /// Number of latches.
    latches: u32,
    /// Number of outputs.
    outputs: u32,
    /// Number of AND gates.
    gates: u32,
}

fn parse_header(s: &str) -> eyre::Result<Header> {
    fn header(s: &str) -> IResult<&str, (u32, u32, u32, u32, u32)> {
        let (s, _) = tag("aag")(s)?;
        let (s, m) = preceded(space1, u32_parser)(s)?;
        let (s, i) = preceded(space1, u32_parser)(s)?;
        let (s, l) = preceded(space1, u32_parser)(s)?;
        let (s, o) = preceded(space1, u32_parser)(s)?;
        let (s, a) = preceded(space1, u32_parser)(s)?;
        Ok((s, (m, i, l, o, a)))
    }

    let (s, (m, i, l, o, a)) = header(s).map_err(|e| e.to_owned())?;
    ensure!(s.is_empty(), "Extra data after header: {}", s);
    ensure!(
        m >= i + l + a,
        "Invalid header {:?}: M < I + L + A",
        (m, i, l, o, a)
    );
    Ok(Header {
        max: m,
        inputs: i,
        latches: l,
        outputs: o,
        gates: a,
    })
}

fn parse_input(s: &str) -> eyre::Result<AigInput> {
    fn input(s: &str) -> IResult<&str, u32> {
        let (s, lit) = u32_parser(s)?;
        Ok((s, lit))
    }

    let (s, lit) = input(s).map_err(|e| e.to_owned())?;
    ensure!(s.is_empty(), "Extra data after input: {}", s);
    ensure!(lit & 1 == 0, "Input must be even: {}", lit);
    let id = lit >> 1;
    Ok(AigInput { id })
}

fn parse_latch(_s: &str) -> IResult<&str, ()> {
    todo!()
}

fn parse_output(s: &str) -> eyre::Result<Ref> {
    fn output(s: &str) -> IResult<&str, u32> {
        let (s, lit) = u32_parser(s)?;
        Ok((s, lit))
    }

    let (s, lit) = output(s).map_err(|e| e.to_owned())?;
    ensure!(s.is_empty(), "Extra data after output: {}", s);
    Ok(Ref::from_u32(lit))
}

fn parse_and_gate(s: &str) -> eyre::Result<AigAndGate> {
    fn and_gate(s: &str) -> IResult<&str, (u32, u32, u32)> {
        let (s, lit) = u32_parser(s)?;
        let (s, left) = preceded(space1, u32_parser)(s)?;
        let (s, right) = preceded(space1, u32_parser)(s)?;
        Ok((s, (lit, left, right)))
    }

    let (s, (lit, left, right)) = and_gate(s).map_err(|e| e.to_owned())?;
    ensure!(s.is_empty(), "Extra data after gate: {}", s);
    ensure!(lit & 1 == 0, "Gate literal must be even: {}", lit);
    let id = lit >> 1;
    let left = Ref::from_u32(left);
    let right = Ref::from_u32(right);
    let args = [left, right];
    Ok(AigAndGate { id, args })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let s = "aag 7 2 0 2 3";
        let header = parse_header(s).unwrap();
        assert_eq!(header.max, 7);
        assert_eq!(header.inputs, 2);
        assert_eq!(header.latches, 0);
        assert_eq!(header.outputs, 2);
        assert_eq!(header.gates, 3);
    }

    #[test]
    fn test_parse_invalid_header_with_max_too_small() {
        let s = "aag 4 2 0 2 3"; // 4 < 2+3
        let res = parse_header(s);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_invalid_header_with_too_large_number() {
        let s = "aag 7 2 0 22222222222222 3"; // value too large for u32
        let res = parse_header(s);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_invalid_header_with_extra_tail() {
        let s = "aag 7 2 0 2 3 "; // note the extra space at the end
        let res = parse_header(s);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_input() {
        let s = "6";
        let input = parse_input(s).unwrap();
        assert_eq!(input.id, 3);
    }

    #[test]
    fn test_parse_invalid_input_with_odd_id() {
        let s = "3";
        let res = parse_input(s);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_output() {
        let s = "4";
        let output = parse_output(s).unwrap();
        assert_eq!(output, Ref::positive(2));
    }

    #[test]
    fn test_parse_output_negated() {
        let s = "7";
        let output = parse_output(s).unwrap();
        assert_eq!(output, Ref::negative(3));
    }

    #[test]
    fn test_parse_and_gate() {
        let s = "8 3 4";
        let gate = parse_and_gate(s).unwrap();
        assert_eq!(gate.id, 4);
        assert_eq!(gate.args[0], Ref::negative(1));
        assert_eq!(gate.args[1], Ref::positive(2));
    }

    #[test]
    fn test_invalid_parse_and_gate_with_odd_id() {
        let s = "9 3 4";
        let res = parse_and_gate(s);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_aig() {
        let aig = Aig::parse_str(
            "\
aag 3 2 0 1 1
2
4
6
6 2 5",
        )
        .unwrap();
        assert_eq!(aig.inputs(), &[AigInput { id: 1 }, AigInput { id: 2 }]);
        assert_eq!(aig.outputs(), &[Ref::positive(3)]);
        assert_eq!(
            aig.gates(),
            &[AigAndGate {
                id: 3,
                args: [Ref::positive(1), Ref::negative(2)]
            }]
        );
    }
}
