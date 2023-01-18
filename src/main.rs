mod cli;

use {clap::Parser, cli::Cli};

fn main() {
    let cli = Cli::parse();

    println!("{:?}", cli);
}
