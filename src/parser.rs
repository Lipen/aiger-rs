use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use color_eyre::eyre::{ensure, eyre};
use nom::bytes::complete::tag;
use nom::character::complete::{space1, u32 as u32_parser};
use nom::sequence::preceded;
use nom::IResult;

use crate::aig::Aig;
use crate::node::{AigAndGate, AigInput, Node};
use crate::reference::Ref;

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
    ands: u32,
}

fn parse_header(s: &str) -> color_eyre::Result<Header> {
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
        ands: a,
    })
}

fn parse_input(s: &str) -> color_eyre::Result<AigInput> {
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

fn parse_output(s: &str) -> color_eyre::Result<Ref> {
    fn output(s: &str) -> IResult<&str, u32> {
        let (s, lit) = u32_parser(s)?;
        Ok((s, lit))
    }

    let (s, lit) = output(s).map_err(|e| e.to_owned())?;
    ensure!(s.is_empty(), "Extra data after output: {}", s);
    Ok(Ref::from_u32(lit))
}

fn parse_and(s: &str) -> color_eyre::Result<AigAndGate> {
    fn and(s: &str) -> IResult<&str, (u32, u32, u32)> {
        let (s, lit) = u32_parser(s)?;
        let (s, left) = preceded(space1, u32_parser)(s)?;
        let (s, right) = preceded(space1, u32_parser)(s)?;
        Ok((s, (lit, left, right)))
    }

    let (s, (lit, left, right)) = and(s).map_err(|e| e.to_owned())?;
    ensure!(s.is_empty(), "Extra data after AND gate: {}", s);
    ensure!(lit & 1 == 0, "AND gate literal must be even: {}", lit);
    let id = lit >> 1;
    let args = [Ref::from_u32(left), Ref::from_u32(right)];
    Ok(AigAndGate { id, args })
}

pub fn parse_aig_iter(mut lines: impl Iterator<Item = String>) -> color_eyre::Result<Aig> {
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

    let mut ands = Vec::with_capacity(header.ands as usize);
    for _ in 0..header.ands {
        let and = parse_and(&lines.next().ok_or_else(|| eyre!("Missing gate"))?)?;
        ensure!(
            and.id <= header.max,
            "And gate id {} is greater than max {}",
            and.id,
            header.max
        );
        ands.push(and);
    }

    let mut mapping: HashMap<u32, Node> = HashMap::new();
    for input in inputs.iter().copied() {
        ensure!(
            !mapping.contains_key(&input.id),
            "Duplicate gate id {}",
            input.id
        );
        mapping.insert(input.id, Node::Input(input));
    }
    for and in ands.iter().copied() {
        ensure!(
            !mapping.contains_key(&and.id),
            "Duplicate gate id {}",
            and.id
        );
        mapping.insert(and.id, Node::AndGate(and));
    }

    let aig = Aig::new(inputs, outputs, ands, mapping);
    Ok(aig)
}

pub fn parse_aig<P: AsRef<Path>>(path: P) -> color_eyre::Result<Aig> {
    let file = File::open(path)?;
    parse_aig_iter(BufReader::new(file).lines().map(|r| r.unwrap()))
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
        assert_eq!(header.ands, 3);
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
        assert_eq!(output.id(), 2);
        assert_eq!(output.is_negated(), false);
    }

    #[test]
    fn test_parse_output_negated() {
        let s = "7";
        let output = parse_output(s).unwrap();
        assert_eq!(output.id(), 3);
        assert_eq!(output.is_negated(), true);
    }

    #[test]
    fn test_parse_and() {
        let s = "8 3 4";
        let and = parse_and(s).unwrap();
        assert_eq!(and.id, 4);
        assert_eq!(and.args[0], Ref::new(1, true));
        assert_eq!(and.args[1], Ref::new(2, false));
    }

    #[test]
    fn test_invalid_parse_and_with_odd_id() {
        let s = "9 3 4";
        let res = parse_and(s);
        assert!(res.is_err());
    }
}
