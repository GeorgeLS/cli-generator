use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    /// The input file to generate cli from
    #[clap(short, long)]
    pub input: String,
    /// The output path to store the generated cli
    #[clap(short, long)]
    pub output: String,
}
