use std::collections::hash_map::Entry;

use {
    crate::parser::{HackPair, Rule},
    anyhow::bail,
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
    writeln!(out, "{:#?}", symbol_table)?;

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::a_instruction => writeln!(out, "A")?,
            Rule::c_instruction => writeln!(out, "C")?,
            Rule::label_definition => writeln!(out, "L")?,
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

    let mut line_number = 0;

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::a_instruction | Rule::c_instruction => line_number += 1,
            Rule::label_definition => {
                let symbol = line
                    .into_inner()
                    .exactly_one()
                    .expect("multiple pairs in Rule::label_definition")
                    .as_str();
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
            Rule::EOI => return Ok(symbol_table),
            _ => unreachable!(),
        }
    }

    unreachable!()
}
