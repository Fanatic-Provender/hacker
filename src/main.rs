#[macro_use]
extern crate pest_derive;

use {clap::Parser as _, pest::Parser as _, parser::HackParser, cli::Cli, std::fs};

mod cli;
mod parser;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let file = fs::read_to_string(cli.file)?;

    let ast = HackParser::parse(parser::Rule::file, &file);
    println!("{:#?}", ast);

    Ok(())
}
