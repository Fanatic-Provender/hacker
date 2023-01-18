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
    std::{
        fs::{self, File},
        io::{self, Write},
    },
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let source_path = &cli.file;

    let source = fs::read_to_string(source_path)?;

    let mut output_file;
    let mut stdout_lock;
    let out: &mut dyn Write = if let Some(path) = &cli.out {
        output_file = File::create(path)?;
        &mut output_file
    } else if !cli.stdout && source_path.extension() == Some("asm".as_ref()) {
        output_file = File::create(source_path.with_extension("hack"))?;
        &mut output_file
    } else {
        stdout_lock = io::stdout().lock();
        &mut stdout_lock
    };

    let ast = HackParser::parse(parser::Rule::file, &source)?
        .exactly_one()
        .expect("multiple pairs matching Rule::file");
    assembler::assemble(ast, out)?;

    Ok(())
}
