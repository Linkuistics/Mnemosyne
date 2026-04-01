use clap::{Parser, Subcommand};
use colored::Colorize;
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
        Commands::Query {
            terms,
            context,
            format,
            max_tokens,
        } => {
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
        Commands::Promote { tags, origin } => {
            let mnemosyne_dir = dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".mnemosyne");
            let store = knowledge::store::KnowledgeStore::new(
                mnemosyne_dir.join("knowledge"),
                mnemosyne_dir.join("archive"),
            );
            let entries = store.load_all()?;

            println!("{}", "Mnemosyne — Promote to Global Knowledge".bold());
            println!();

            println!("Title for this knowledge entry:");
            let mut title = String::new();
            std::io::stdin().read_line(&mut title)?;
            let title = title.trim().to_string();

            let tags: Vec<String> = if let Some(ref t) = tags {
                t.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                println!("Tags (comma-separated):");
                let mut tag_input = String::new();
                std::io::stdin().read_line(&mut tag_input)?;
                tag_input.split(',').map(|s| s.trim().to_string()).collect()
            };

            let origin = origin.unwrap_or_else(|| {
                println!("Origin project:");
                let mut o = String::new();
                std::io::stdin().read_line(&mut o).unwrap();
                o.trim().to_string()
            });

            println!("Knowledge content (end with empty line):");
            let mut body = String::new();
            loop {
                let mut line = String::new();
                std::io::stdin().read_line(&mut line)?;
                if line.trim().is_empty() {
                    break;
                }
                body.push_str(&line);
            }

            let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
            let new_entry = commands::promote::build_new_entry(
                &title,
                &tag_refs,
                knowledge::entry::Confidence::High,
                &origin,
                "manual promotion",
                &body,
            );

            let contradictions = commands::promote::check_contradictions(&entries, &new_entry);
            if !contradictions.is_empty() {
                println!(
                    "\n{}",
                    "⚠ Potential contradictions detected:".yellow().bold()
                );
                for c in &contradictions {
                    println!(
                        "  {} (overlap: {:.0}%)",
                        c.existing.title,
                        c.overlap_score * 100.0
                    );
                }
                println!("\n[s]upersede  [c]oexist  [d]iscard  [r]efine");
                let mut choice = String::new();
                std::io::stdin().read_line(&mut choice)?;
                if let Some('d') = choice.trim().chars().next() {
                    println!("Discarded.");
                    return Ok(());
                }
            }

            let axis = commands::promote::suggest_axis(&new_entry.tags);
            let filename = commands::promote::title_to_filename(&title);
            let mut entry = new_entry;
            store.create_entry(axis, &filename, &mut entry)?;
            println!("\n✓ Promoted to knowledge/{}/{}", axis, filename);
        }
        Commands::Curate => {
            let mnemosyne_dir = dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".mnemosyne");
            let store = knowledge::store::KnowledgeStore::new(
                mnemosyne_dir.join("knowledge"),
                mnemosyne_dir.join("archive"),
            );
            let entries = store.load_all()?;
            commands::curate::run_curate(&store, &entries)?;
        }
        Commands::Explore => {
            let mnemosyne_dir = dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".mnemosyne");
            let store = knowledge::store::KnowledgeStore::new(
                mnemosyne_dir.join("knowledge"),
                mnemosyne_dir.join("archive"),
            );
            let entries = store.load_all()?;
            commands::explore::run_explore(&store, &entries)?;
        }
        Commands::Install { adapter } => match adapter.as_str() {
            "claude-code" => {
                let plugin_target = dirs::home_dir()
                    .expect("Could not determine home directory")
                    .join(".claude/plugins/observational-memory");

                let exe_dir = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()));
                let source_candidates = [
                    exe_dir.as_ref().map(|d| d.join("../adapters/claude-code")),
                    Some(std::path::PathBuf::from("adapters/claude-code")),
                ];

                let source = source_candidates
                    .iter()
                    .flatten()
                    .find(|p| p.exists())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Could not find adapter files. Run from the Mnemosyne repo directory."
                        )
                    })?;

                commands::install::run_install_claude_code(source, &plugin_target)?;
                println!(
                    "✓ Claude Code plugin installed to {}",
                    plugin_target.display()
                );
            }
            other => {
                println!("Unknown adapter: {}. Available: claude-code", other);
            }
        },
        Commands::Status => {
            let mnemosyne_dir = dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".mnemosyne");
            let store = knowledge::store::KnowledgeStore::new(
                mnemosyne_dir.join("knowledge"),
                mnemosyne_dir.join("archive"),
            );
            let entries = store.load_all()?;
            print!(
                "{}",
                commands::status::run_status(&entries, &mnemosyne_dir)?
            );
        }
    }
    Ok(())
}
