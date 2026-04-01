use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod contradiction;
mod context;
mod corpus;
mod report;
mod retrieval;

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    let corpus = corpus::Corpus::load(&cli.corpus)?;

    let retrieval_metrics = retrieval::evaluate_retrieval(&corpus, cli.k);
    let contradiction_metrics = contradiction::evaluate_contradictions(&corpus, 0.5);
    let context_metrics = context::evaluate_context(&corpus);

    let sweep = if cli.sweep {
        Some(contradiction::sweep_thresholds(&corpus))
    } else {
        None
    };

    if cli.json {
        println!(
            "{}",
            report::format_json(
                &retrieval_metrics,
                &contradiction_metrics,
                &context_metrics,
                &sweep
            )
        );
    } else {
        print!(
            "{}",
            report::format_human(
                &retrieval_metrics,
                &contradiction_metrics,
                &context_metrics,
                &sweep,
                cli.verbose
            )
        );
    }

    Ok(())
}
