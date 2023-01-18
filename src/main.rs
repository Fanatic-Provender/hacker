mod assembler;
mod cli;
mod parser;

#[macro_use]
extern crate pest_derive;

use {
    crate::{cli::Cli, parser::HackParser},
    clap::Parser as _,
    itertools::Itertools,
    pest::Parser as _,
    std::{fs, io},
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let file = fs::read_to_string(cli.file)?;

    let ast = HackParser::parse(parser::Rule::file, &file)?
        .exactly_one()
        .expect("multiple pairs matching Rule::file");
    assembler::assemble(ast, io::stdout().lock())?;

    Ok(())
}
