#[derive(Parser)]
#[grammar = "hack.pest"]
pub struct HackParser;

pub type HackPair<'i> = pest::iterators::Pair<'i, Rule>;
