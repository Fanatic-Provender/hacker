use std::collections::hash_map::Entry;

use {
    crate::parser::{HackPair, Rule},
    anyhow::{anyhow, bail},
    itertools::Itertools,
    std::{collections::HashMap, io::Write},
};

#[derive(Clone, Debug)]
pub struct SymbolData {
    pub value: usize,
    pub is_predefined: bool,
}

type SymbolTable = HashMap<String, SymbolData>;

const RESERVED_REGISTERS: usize = 16;
const INSTRUCTION_WIDTH: usize = 16;
const ADDRESS_SPACE_SIZE: usize = 32768;
static KEYWORDS: &[(&str, usize)] = &[
    ("SP", 0),
    ("LCL", 1),
    ("ARG", 2),
    ("THIS", 3),
    ("THAT", 4),
    ("SCREEN", 16384),
    ("KBD", 24576),
];

pub fn assemble<W: Write>(file: HackPair, mut out: W) -> anyhow::Result<()> {
    let symbol_table = scan_symbols(file.clone())?;

    dbg!();

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::a_instruction => {
                let spec = line.into_inner().exactly_one().unwrap();

                let value = match spec.as_rule() {
                    Rule::constant => {
                        let spec = spec.as_str();
                        let value = spec
                            .parse()
                            .map_err(|_| anyhow!("invalid constant '{}'", spec))?;
                        if (0..ADDRESS_SPACE_SIZE).contains(&value) {
                            value
                        } else {
                            bail!("invalid constant '{}'", spec)
                        }
                    }
                    Rule::symbol => {
                        symbol_table
                            .get(spec.as_str())
                            .expect("incomplete symbol table")
                            .value
                    }
                    _ => unreachable!(),
                };

                writeln!(out, "{value:0width$b}", width = INSTRUCTION_WIDTH)?;
            }
            Rule::c_instruction => writeln!(out, "[TODO] C instruction")?,
            Rule::label_definition => {}
            Rule::EOI => return Ok(()),
            _ => unreachable!(),
        }
    }

    unreachable!()
}

pub fn scan_symbols(file: HackPair) -> anyhow::Result<SymbolTable> {
    let mut symbol_table: SymbolTable = (0..RESERVED_REGISTERS)
        .map(|i| {
            (
                format!("R{i}"),
                SymbolData {
                    value: i,
                    is_predefined: true,
                },
            )
        })
        .chain(KEYWORDS.iter().map(|&(s, v)| {
            (
                s.to_owned(),
                SymbolData {
                    value: v,
                    is_predefined: true,
                },
            )
        }))
        .collect();

    // scan labels
    let mut line_number = 0;

    for line in file.clone().into_inner() {
        match line.as_rule() {
            Rule::a_instruction | Rule::c_instruction => {
                if line_number >= ADDRESS_SPACE_SIZE {
                    bail!("too many lines");
                }
                line_number += 1;
            }
            Rule::label_definition => {
                let symbol = line.into_inner().exactly_one().unwrap().as_str();
                match symbol_table.entry(symbol.to_owned()) {
                    Entry::Occupied(entry) => {
                        if entry.get().is_predefined {
                            bail!("symbol '{symbol}' is predefined");
                        } else {
                            bail!("symbol '{symbol}' is already defined");
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(SymbolData {
                            value: line_number,
                            is_predefined: false,
                        });
                    }
                }
            }
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }

    dbg!();

    // scan variables
    let mut stack_top = RESERVED_REGISTERS;

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::a_instruction => {
                dbg!(line.line_col());

                let inner = line.into_inner().exactly_one().unwrap();

                match inner.as_rule() {
                    Rule::constant => {}
                    Rule::symbol => {
                        let symbol = inner.as_str();
                        if symbol_table.contains_key(symbol) {
                            continue;
                        }
                        if stack_top >= ADDRESS_SPACE_SIZE {
                            bail!("too many variables");
                        }

                        symbol_table.insert(
                            symbol.to_owned(),
                            SymbolData {
                                value: stack_top,
                                is_predefined: false,
                            },
                        );
                        stack_top += 1;
                    }
                    _ => unreachable!(),
                }
            }
            Rule::c_instruction | Rule::label_definition => {}
            Rule::EOI => return Ok(symbol_table),
            _ => unreachable!(),
        }
    }

    unreachable!()
}
