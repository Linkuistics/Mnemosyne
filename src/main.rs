use clap::{Parser, Subcommand};
use mnemosyne::commands;
use mnemosyne::config;
use mnemosyne::context;
use mnemosyne::knowledge;

#[derive(Parser)]
#[command(name = "mnemosyne", about = "Global developer knowledge system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create ~/.mnemosyne/ with default structure
    Init {
        /// Clone an existing knowledge repo
        #[arg(long)]
        from: Option<String>,
    },
    /// Search global knowledge
    Query {
        /// Search terms
        terms: Vec<String>,
        /// Infer context from current project
        #[arg(long)]
        context: bool,
        /// Output format: markdown, json, plain
        #[arg(long, default_value = "markdown")]
        format: String,
        /// Limit output to fit within token budget
        #[arg(long)]
        max_tokens: Option<usize>,
    },
    /// Promote a learning to global knowledge
    Promote {
        /// Tags for the new entry
        #[arg(long)]
        tags: Option<String>,
        /// Origin project name
        #[arg(long)]
        origin: Option<String>,
    },
    /// Reflective curation session
    Curate,
    /// Interactive knowledge exploration
    Explore,
    /// Install adapter plugin
    Install {
        /// Adapter name (e.g., "claude-code")
        adapter: String,
    },
    /// Knowledge base status summary
    Status,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { from } => {
            let mnemosyne_dir = dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".mnemosyne");
            commands::init::run_init(&mnemosyne_dir, from.as_deref())?;
            println!("Mnemosyne initialized at {}", mnemosyne_dir.display());
        }
        Commands::Query { terms, context, format, max_tokens } => {
            let mnemosyne_dir = dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".mnemosyne");
            let store = knowledge::store::KnowledgeStore::new(
                mnemosyne_dir.join("knowledge"),
                mnemosyne_dir.join("archive"),
            );
            let entries = store.load_all()?;

            let output_format = commands::query::OutputFormat::from_str(&format);
            let max_results = max_tokens.map(|t| t / 500).unwrap_or(10);

            if context {
                let config = config::Config::load(&mnemosyne_dir)?;
                let detector = context::detect::ProjectDetector::new(&config);
                let signals = detector.detect(&std::env::current_dir()?)?;
                let mapper = context::mapping::SignalMapper::new(&config);
                let tags = mapper.map_signals(&signals);

                let opts = commands::query::QueryOptions {
                    terms: vec![],
                    tags,
                    format: output_format,
                    max_results,
                };
                print!("{}", commands::query::run_query(&entries, &opts)?);
            } else {
                let opts = commands::query::QueryOptions {
                    terms,
                    tags: vec![],
                    format: output_format,
                    max_results,
                };
                print!("{}", commands::query::run_query(&entries, &opts)?);
            }
        }
        Commands::Promote { .. } => println!("promote: not yet implemented"),
        Commands::Curate => println!("curate: not yet implemented"),
        Commands::Explore => println!("explore: not yet implemented"),
        Commands::Install { .. } => println!("install: not yet implemented"),
        Commands::Status => println!("status: not yet implemented"),
    }
    Ok(())
}
