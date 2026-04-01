use std::path::PathBuf;
use clap::Parser;

mod corpus;
mod retrieval;
mod contradiction;
mod context;
mod report;

#[derive(Parser)]
#[command(name = "mnemosyne-eval")]
struct Cli {
    /// Path to corpus directory
    #[arg(long, default_value = "corpus")]
    corpus: PathBuf,

    /// Top-k for retrieval metrics
    #[arg(long, default_value = "5")]
    k: usize,

    /// Run contradiction threshold sweep
    #[arg(long)]
    sweep: bool,

    /// Per-query and per-pair breakdown
    #[arg(long)]
    verbose: bool,

    /// Output in JSON format
    #[arg(long)]
    json: bool,
}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    println!("mnemosyne-eval: scaffold OK");
    Ok(())
}
