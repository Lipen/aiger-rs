use std::fmt::{Display, Formatter};
use std::io::{BufRead, Lines};
use std::str::FromStr;

use eyre::{eyre, WrapErr};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Literal(u32);

impl Literal {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn from_variable(variable: u32, is_negated: bool) -> Self {
        Self::new((variable << 1) + is_negated as u32)
    }

    pub const fn raw(&self) -> u32 {
        self.0
    }
    pub const fn index(&self) -> u32 {
        self.0 >> 1
    }
    pub const fn is_negated(&self) -> bool {
        self.0 & 1 != 0
    }
}

/// AIGER header.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Header {
    /// The maximum variable index.
    pub m: usize,
    /// The number of inputs.
    pub i: usize,
    /// The number of latches.
    pub l: usize,
    /// The number of outputs.
    pub o: usize,
    /// The number of AND gates.
    pub a: usize,
}

const TAG: &str = "aag";

impl FromStr for Header {
    type Err = eyre::Error;

    fn from_str(line: &str) -> eyre::Result<Self> {
        let mut components = line.split(' ');

        let tag = components.next().ok_or_else(|| eyre!("Tag is missing"))?;
        if tag != TAG {
            return Err(eyre!("Invalid tag '{}', expected '{}'", tag, TAG));
        }

        let mut components = components.map(|s| {
            s.parse::<usize>()
                .map_err(|_| eyre!("Invalid component '{}', expected non-negative number", s))
        });

        let mut next_component = || {
            components
                .next()
                .ok_or_else(|| eyre!("Not enough components, expected 'aag m i l o a'"))?
        };
        let m = next_component()?;
        let i = next_component()?;
        let l = next_component()?;
        let o = next_component()?;
        let a = next_component()?;

        if components.next().is_some() {
            // There are more components than expected.
            return Err(eyre!("Too many components, expected 'aag m i l o a'"));
        }

        Ok(Header { m, i, l, o, a })
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            TAG, self.m, self.i, self.l, self.o, self.a
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SymbolType {
    Input,
    Latch,
    Output,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Record {
    Input {
        id: Literal,
    },
    Latch {
        /// The current state.
        id: Literal,
        /// The next state.
        next: Literal,
    },
    Output {
        id: Literal,
    },
    AndGate {
        id: Literal,
        inputs: [Literal; 2],
    },
    Symbol {
        type_spec: SymbolType,
        position: usize,
        symbol: String,
    },
}

impl Record {
    fn parse_input(literals: &[Literal]) -> eyre::Result<Record> {
        match literals {
            &[id] => Ok(Record::Input { id }),
            _ => Err(eyre!(
                "Invalid number of literals for input: expected 1, got {}",
                literals.len()
            )),
        }
    }

    fn parse_latch(literals: &[Literal]) -> eyre::Result<Record> {
        match literals {
            &[id, next] => Ok(Record::Latch { id, next }),
            _ => Err(eyre!(
                "Invalid number of literals for latch: expected 2, got {}",
                literals.len()
            )),
        }
    }

    fn parse_output(literals: &[Literal]) -> eyre::Result<Record> {
        match literals {
            &[id] => Ok(Record::Output { id }),
            _ => Err(eyre!(
                "Invalid number of literals for output: expected 1, got {}",
                literals.len()
            )),
        }
    }

    fn parse_and_gate(literals: &[Literal]) -> eyre::Result<Record> {
        match literals {
            &[id, left, right] => Ok(Record::AndGate {
                id,
                inputs: [left, right],
            }),
            _ => Err(eyre!(
                "Invalid number of literals for and gate: expected 3, got {}",
                literals.len()
            )),
        }
    }

    fn parse_symbol(line: &str) -> eyre::Result<Record> {
        let (type_spec, rest) = line.split_at(1);
        let type_spec = match type_spec {
            "i" => SymbolType::Input,
            "l" => SymbolType::Latch,
            "o" => SymbolType::Output,
            _ => {
                return Err(eyre!(
                    "Invalid type '{}', expected 'i', 'l' or 'o'",
                    type_spec
                ))
            }
        };

        let space_position = rest.find(' ').ok_or_else(|| eyre!("Expected space"))?;
        let (position, rest) = rest.split_at(space_position);
        let position = position
            .parse::<usize>()
            .map_err(|_| eyre!("Could not parse position '{}' as usize", position))?;

        let symbol = &rest[1..];
        if symbol.is_empty() {
            return Err(eyre!("Symbol name is empty"));
        }
        Ok(Record::Symbol {
            type_spec,
            position,
            symbol: symbol.to_string(),
        })
    }

    fn validate(self, header: &Header) -> eyre::Result<Self> {
        match &self {
            Record::Input { id } => {
                if id.index() == 0 {
                    return Err(eyre!("Input index must be non-zero"));
                }
                if id.index() > header.m as u32 {
                    return Err(eyre!(
                        "Input {} is out of range (1..{})",
                        id.index(),
                        header.m
                    ));
                }
                if id.is_negated() {
                    return Err(eyre!("Input {} is inverted", id.index()));
                }
            }
            Record::Latch { id, next } => {
                if id.index() == 0 {
                    return Err(eyre!("Latch index must be non-zero"));
                }
                if next.index() == 0 {
                    return Err(eyre!("Latch next index must be non-zero"));
                }
                if id.index() > header.m as u32 {
                    return Err(eyre!(
                        "Latch {} is out of range (1..{})",
                        id.index(),
                        header.m
                    ));
                }
                if next.index() > header.m as u32 {
                    return Err(eyre!(
                        "Latch next {} is out of range (1..{})",
                        next.index(),
                        header.m
                    ));
                }
                if id.is_negated() {
                    return Err(eyre!("Latch {} is inverted", id.index()));
                }
            }
            Record::Output { id } => {
                if id.index() > header.m as u32 {
                    return Err(eyre!(
                        "Output {} is out of range (1..{})",
                        id.index(),
                        header.m
                    ));
                }
            }
            Record::AndGate { id, inputs } => {
                if id.index() > header.m as u32 {
                    return Err(eyre!(
                        "And gate {} is out of range (1..{})",
                        id.index(),
                        header.m
                    ));
                }
                let [left, right] = inputs;
                if left.index() > header.m as u32 {
                    return Err(eyre!(
                        "And gate left {} is out of range (1..{})",
                        left.index(),
                        header.m
                    ));
                }
                if right.index() > header.m as u32 {
                    return Err(eyre!(
                        "And gate right {} is out of range (1..{})",
                        right.index(),
                        header.m
                    ));
                }
            }
            Record::Symbol { .. } => {}
        }

        Ok(self)
    }
}

/// A reader for AIGER files.
pub struct Reader<R> {
    lines: Lines<R>,
    header: Header,
}

impl<R: BufRead> Reader<R> {
    pub fn new(reader: R) -> eyre::Result<Reader<R>> {
        let mut lines = reader.lines();

        let header_line = lines
            .next()
            .ok_or_else(|| eyre!("Header line is missing"))??;
        let header = header_line
            .parse::<Header>()
            .wrap_err_with(|| format!("Invalid header '{}'", header_line))?;

        Ok(Reader { lines, header })
    }
}

impl<R> Reader<R> {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn records(self) -> RecordsIter<R> {
        RecordsIter::new(self.lines, self.header)
    }
}

/// An iterator over the records in an AIGER file.
pub struct RecordsIter<R> {
    lines: Lines<R>,
    header: Header,
    remaining_inputs: usize,
    remaining_latches: usize,
    remaining_outputs: usize,
    remaining_and_gates: usize,
    comment: bool,
}

impl<R> RecordsIter<R> {
    fn new(lines: Lines<R>, header: Header) -> RecordsIter<R> {
        RecordsIter {
            lines,
            remaining_inputs: header.i,
            remaining_latches: header.l,
            remaining_outputs: header.o,
            remaining_and_gates: header.a,
            comment: false,
            header, // last to allow move
        }
    }

    fn read_record(&mut self, line: &str) -> eyre::Result<Record> {
        fn get_literals(line: &str) -> eyre::Result<Vec<Literal>> {
            let mut literals = Vec::new();
            for part in line.split(' ') {
                let lit = part
                    .parse::<u32>()
                    .map_err(|_| eyre!("Invalid literal '{}', expected u32 number", part))?;
                literals.push(Literal::new(lit));
            }
            Ok(literals)
        }

        if self.remaining_inputs > 0 {
            self.remaining_inputs -= 1;
            Record::parse_input(&get_literals(line)?)
                .wrap_err_with(|| format!("Invalid input '{}'", line))
        } else if self.remaining_latches > 0 {
            self.remaining_latches -= 1;
            Record::parse_latch(&get_literals(line)?)
                .wrap_err_with(|| format!("Invalid latch '{}'", line))
        } else if self.remaining_outputs > 0 {
            self.remaining_outputs -= 1;
            Record::parse_output(&get_literals(line)?)
                .wrap_err_with(|| format!("Invalid output '{}'", line))
        } else if self.remaining_and_gates > 0 {
            self.remaining_and_gates -= 1;
            Record::parse_and_gate(&get_literals(line)?)
                .wrap_err_with(|| format!("Invalid and gate '{}'", line))
        } else {
            Record::parse_symbol(line).wrap_err_with(|| format!("Invalid symbol '{}'", line))
        }
    }
}

impl<R: BufRead> Iterator for RecordsIter<R> {
    type Item = eyre::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.comment {
            return None;
        }

        let line = match self.lines.next() {
            Some(Ok(line)) => line,
            Some(Err(e)) => return Some(Err(e.into())),
            None => return None,
        };

        if line.starts_with('c') {
            self.comment = true;
            return None;
        }

        Some(
            self.read_record(&line)
                .wrap_err_with(|| format!("Invalid record '{}'", line))
                .and_then(|r| r.validate(&self.header)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    #[test]
    fn test_parse_header() {
        let input = "aag 5 2 0 1 2";
        let header = input.parse::<Header>().unwrap();
        assert_eq!(header.m, 5);
        assert_eq!(header.i, 2);
        assert_eq!(header.l, 0);
        assert_eq!(header.o, 1);
        assert_eq!(header.a, 2);
    }

    fn make_reader(input: &str) -> eyre::Result<Reader<&[u8]>> {
        Reader::new(input.as_bytes())
    }

    #[test]
    fn test_reader_single_input() {
        let input = indoc! {"
            aag 1 1 0 0 0
            2
        "};
        let reader = make_reader(input).unwrap();

        let header = reader.header();
        assert_eq!(header.m, 1);
        assert_eq!(header.i, 1);
        assert_eq!(header.l, 0);
        assert_eq!(header.o, 0);
        assert_eq!(header.a, 0);

        let mut records = reader.records();
        let mut next = || records.next().map(|x| x.unwrap());
        assert_eq!(
            next(),
            Some(Record::Input {
                id: Literal::new(2)
            })
        );
        assert_eq!(next(), None);
    }

    #[test]
    fn test_reader_and_gate() {
        let input = indoc! {"
            aag 3 2 0 1 1
            2
            4
            6
            6 2 4
        "};
        let reader = make_reader(input).unwrap();

        let header = reader.header();
        assert_eq!(header.m, 3);
        assert_eq!(header.i, 2);
        assert_eq!(header.l, 0);
        assert_eq!(header.o, 1);
        assert_eq!(header.a, 1);

        let mut records = reader.records();
        let mut next = || records.next().map(|x| x.unwrap());
        assert_eq!(
            next(),
            Some(Record::Input {
                id: Literal::new(2)
            })
        );
        assert_eq!(
            next(),
            Some(Record::Input {
                id: Literal::new(4)
            })
        );
        assert_eq!(
            next(),
            Some(Record::Output {
                id: Literal::new(6)
            })
        );
        assert_eq!(
            next(),
            Some(Record::AndGate {
                id: Literal::new(6),
                inputs: [Literal::new(2), Literal::new(4)]
            })
        );
        assert_eq!(next(), None);
    }

    #[test]
    fn test_reader_or_gate() {
        let input = indoc! {"
            aag 3 2 0 1 1
            2
            4
            7
            6 3 5
        "};
        let reader = make_reader(input).unwrap();

        let header = reader.header();
        assert_eq!(header.m, 3);
        assert_eq!(header.i, 2);
        assert_eq!(header.l, 0);
        assert_eq!(header.o, 1);
        assert_eq!(header.a, 1);

        let mut records = reader.records();
        let mut next = || records.next().map(|x| x.unwrap());
        assert_eq!(
            next(),
            Some(Record::Input {
                id: Literal::new(2)
            })
        );
        assert_eq!(
            next(),
            Some(Record::Input {
                id: Literal::new(4)
            })
        );
        assert_eq!(
            next(),
            Some(Record::Output {
                id: Literal::new(7)
            })
        );
        assert_eq!(
            next(),
            Some(Record::AndGate {
                id: Literal::new(6),
                inputs: [Literal::new(3), Literal::new(5)]
            })
        );
        assert_eq!(next(), None);
    }
}
