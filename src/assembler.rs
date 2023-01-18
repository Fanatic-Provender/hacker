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
            Rule::c_instruction => {
                let mut dest = "";
                let mut comp = "";
                let mut jump = "";

                for spec in line.into_inner() {
                    match spec.as_rule() {
                        Rule::dest => dest = spec.as_str(),
                        Rule::comp => comp = spec.as_str(),
                        Rule::jump => jump = spec.as_str(),
                        _ => unreachable!(),
                    }
                }

                #[rustfmt::skip]
                let dest_code = match (dest.contains('A'), dest.contains('D'), dest.contains('M')) {
                    (false, false, false) => "000",
                    (false, false,  true) => "001",
                    (false,  true, false) => "010",
                    (false,  true,  true) => "011",
                    ( true, false, false) => "100",
                    ( true, false,  true) => "101",
                    ( true,  true, false) => "110",
                    ( true,  true,  true) => "111",
                };
                #[rustfmt::skip]
                let a_comp_code = match comp {
                      "0" => "0101010",
                      "1" => "0111111",
                     "-1" => "0111010",
                      "D" => "0001100",
                      "A" => "0110000",
                      "M" => "1110000",
                     "!D" => "0001101",
                     "!A" => "0110001",
                     "!M" => "1110001",
                     "-D" => "0001111",
                     "-A" => "0110011",
                     "-M" => "1110011",
                    "D+1" => "0011111",
                    "A+1" => "0110111",
                    "M+1" => "1110111",
                    "D-1" => "0001110",
                    "A-1" => "0110010",
                    "M-1" => "1110010",
                    "D+A" => "0000010",
                    "D+M" => "1000010",
                    "D-A" => "0010011",
                    "D-M" => "1010011",
                    "A-D" => "0000111",
                    "M-D" => "1000111",
                    "D&A" => "0000000",
                    "D&M" => "1000000",
                    "D|A" => "0010101",
                    "D|M" => "1010101",
                        _ => bail!("invalid computation '{}'", comp)
                };
                #[rustfmt::skip]
                let jump_code = match jump {
                       "" => "000",
                    "JGT" => "001",
                    "JEQ" => "010",
                    "JGE" => "011",
                    "JLT" => "100",
                    "JNE" => "101",
                    "JLE" => "110",
                    "JMP" => "111",
                        _ => bail!("invalid jump '{}'", jump)
                };

                writeln!(out, "111{}{}{}", a_comp_code, dest_code, jump_code)?;
            }
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

    // scan variables
    let mut stack_top = RESERVED_REGISTERS;

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::a_instruction => {
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
