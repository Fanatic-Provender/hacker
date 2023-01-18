use {clap::Parser, std::path::PathBuf};

#[derive(Debug, Parser)]
pub struct Cli {
    pub file: PathBuf,
    #[arg(short, long)]
    pub out: Option<PathBuf>,
    #[arg(short, long)]
    pub stdout: bool,
}
