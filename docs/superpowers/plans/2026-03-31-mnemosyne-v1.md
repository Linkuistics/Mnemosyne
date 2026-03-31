# Mnemosyne v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Mnemosyne CLI and Claude Code plugin — a two-tier developer knowledge system with a Rust CLI for managing global knowledge and an updated observational-memory plugin for Claude Code integration.

**Architecture:** A Rust CLI binary (`mnemosyne`) manages a Git-backed knowledge store at `~/.mnemosyne/`. Knowledge entries are markdown files with YAML frontmatter, indexed in-memory via tag matching. The Claude Code plugin shells out to the CLI for global operations and degrades gracefully when the CLI isn't installed.

**Tech Stack:** Rust (1.93.1), clap (CLI), serde/serde_yaml/serde_json (serialization), chrono (dates), ignore (directory walking), colored (terminal output), anyhow/thiserror (errors)

---

## File Map

### Rust CLI (`src/`)

| File | Responsibility |
|------|---------------|
| `src/main.rs` | CLI entry point, clap command definitions |
| `src/lib.rs` | Re-exports for library use |
| `src/knowledge/mod.rs` | Module declarations |
| `src/knowledge/entry.rs` | `Entry`, `Confidence`, `Origin` types + frontmatter parsing/serialization |
| `src/knowledge/store.rs` | `KnowledgeStore` — read/write/archive entries from filesystem |
| `src/knowledge/tags.rs` | `TagMatcher` — tag overlap, scoring, similarity |
| `src/knowledge/index.rs` | `KnowledgeIndex` trait + `FileIndex` implementation |
| `src/evolution/mod.rs` | Module declarations |
| `src/evolution/contradiction.rs` | `ContradictionDetector` — find conflicting entries by tag overlap |
| `src/evolution/supersede.rs` | `supersede_entry()` — move old content to Superseded section |
| `src/evolution/divergence.rs` | `DivergenceDetector` — find implicit divergence across projects |
| `src/context/mod.rs` | Module declarations |
| `src/context/detect.rs` | `ProjectDetector` — scan project root for language/tool signals |
| `src/context/mapping.rs` | `SignalMapper` — convert detected signals to tags |
| `src/config.rs` | `Config` type — parse `~/.mnemosyne/config.yml` |
| `src/commands/mod.rs` | Module declarations |
| `src/commands/init.rs` | `mnemosyne init` implementation |
| `src/commands/status.rs` | `mnemosyne status` implementation |
| `src/commands/query.rs` | `mnemosyne query` implementation |
| `src/commands/promote.rs` | `mnemosyne promote` implementation |
| `src/commands/curate.rs` | `mnemosyne curate` implementation |
| `src/commands/explore.rs` | `mnemosyne explore` implementation |
| `src/commands/install.rs` | `mnemosyne install claude-code` implementation |

### Tests (`tests/`)

| File | What it tests |
|------|--------------|
| `tests/entry_test.rs` | Frontmatter parsing, serialization, round-trip |
| `tests/store_test.rs` | Loading entries from filesystem, saving, archiving |
| `tests/tags_test.rs` | Tag matching, overlap scoring |
| `tests/index_test.rs` | Search, find_by_tags, find_contradictions |
| `tests/contradiction_test.rs` | Contradiction detection logic |
| `tests/supersede_test.rs` | Supersession formatting and file mutation |
| `tests/divergence_test.rs` | Divergence detection across origins |
| `tests/detect_test.rs` | Project signal detection from marker files |
| `tests/mapping_test.rs` | Signal-to-tag mapping |
| `tests/config_test.rs` | Config parsing and defaults |
| `tests/init_test.rs` | Init command creates correct directory structure |
| `tests/query_test.rs` | Query command integration test |
| `tests/promote_test.rs` | Promote command integration test |
| `tests/install_test.rs` | Install command copies plugin files correctly |

### Claude Code Plugin (`adapters/claude-code/`)

| File | Responsibility |
|------|---------------|
| `adapters/claude-code/plugin.json` | Plugin manifest |
| `adapters/claude-code/skills/begin-work.md` | Start work with global knowledge loading |
| `adapters/claude-code/skills/reflect.md` | Reflection with global promotion |
| `adapters/claude-code/skills/create-plan.md` | Plan creation (unchanged from current) |
| `adapters/claude-code/skills/setup-knowledge.md` | Project scaffolding + mnemosyne init |
| `adapters/claude-code/skills/curate-global.md` | Global curation session |
| `adapters/claude-code/skills/promote-global.md` | Ad-hoc global promotion |
| `adapters/claude-code/skills/explore-knowledge.md` | Interactive knowledge exploration |
| `adapters/claude-code/references/observational-memory-guide.md` | Core guide |
| `adapters/claude-code/references/plan-format.md` | Plan format reference |
| `adapters/claude-code/references/coding-conventions.md` | Coding conventions |
| `adapters/claude-code/references/global-knowledge-guide.md` | Global knowledge reference |

### Documentation (`docs/`)

| File | Responsibility |
|------|---------------|
| `README.md` | Project overview, quick start, philosophy |
| `docs/user-guide.md` | Complete walkthrough |
| `docs/reference.md` | CLI command reference |
| `docs/knowledge-format.md` | Knowledge file format spec |
| `docs/evolution-guide.md` | Knowledge evolution philosophy and mechanics |
| `docs/configuration.md` | Config file reference |
| `docs/plugin-development.md` | Guide for building harness adapters |
| `docs/research-sources.md` | Annotated research bibliography |

### Default Config (bundled with `init`)

| File | Responsibility |
|------|---------------|
| `defaults/config.yml` | Default configuration with language profiles |
| `defaults/gitignore` | Default .gitignore for ~/.mnemosyne/ |

---

## Task 1: Project Scaffold and Tooling

**Files:**
- Create: `Cargo.toml`
- Create: `.tool-versions`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `.gitignore`

- [ ] **Step 1: Create .tool-versions**

```
rust 1.93.1
```

- [ ] **Step 2: Create Cargo.toml**

```toml
[package]
name = "mnemosyne"
version = "0.1.0"
edition = "2021"
description = "Global developer knowledge system for LLM-driven development"
license = "MIT"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
colored = "3"
ignore = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
thiserror = "2"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 3: Create src/main.rs with minimal clap structure**

```rust
use clap::{Parser, Subcommand};

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
        Commands::Init { .. } => println!("init: not yet implemented"),
        Commands::Query { .. } => println!("query: not yet implemented"),
        Commands::Promote { .. } => println!("promote: not yet implemented"),
        Commands::Curate => println!("curate: not yet implemented"),
        Commands::Explore => println!("explore: not yet implemented"),
        Commands::Install { .. } => println!("install: not yet implemented"),
        Commands::Status => println!("status: not yet implemented"),
    }
    Ok(())
}
```

- [ ] **Step 4: Create src/lib.rs**

```rust
pub mod config;
pub mod knowledge;
pub mod evolution;
pub mod context;
pub mod commands;
```

- [ ] **Step 5: Create module stub files**

Create these empty module files so the project compiles:
- `src/knowledge/mod.rs` — `pub mod entry; pub mod store; pub mod tags; pub mod index;`
- `src/knowledge/entry.rs` — empty
- `src/knowledge/store.rs` — empty
- `src/knowledge/tags.rs` — empty
- `src/knowledge/index.rs` — empty
- `src/evolution/mod.rs` — `pub mod contradiction; pub mod supersede; pub mod divergence;`
- `src/evolution/contradiction.rs` — empty
- `src/evolution/supersede.rs` — empty
- `src/evolution/divergence.rs` — empty
- `src/context/mod.rs` — `pub mod detect; pub mod mapping;`
- `src/context/detect.rs` — empty
- `src/context/mapping.rs` — empty
- `src/config.rs` — empty
- `src/commands/mod.rs` — `pub mod init; pub mod status; pub mod query; pub mod promote; pub mod curate; pub mod explore; pub mod install;`
- `src/commands/init.rs` — empty
- `src/commands/status.rs` — empty
- `src/commands/query.rs` — empty
- `src/commands/promote.rs` — empty
- `src/commands/curate.rs` — empty
- `src/commands/explore.rs` — empty
- `src/commands/install.rs` — empty

- [ ] **Step 6: Create .gitignore**

```
/target
```

- [ ] **Step 7: Verify it compiles**

Run: `cargo build`
Expected: Compiles with no errors (warnings about unused modules are fine)

- [ ] **Step 8: Verify CLI works**

Run: `cargo run -- --help`
Expected: Shows help with all subcommands listed

Run: `cargo run -- init`
Expected: Prints "init: not yet implemented"

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock .tool-versions .gitignore src/
git commit -m "feat: scaffold Rust CLI with clap command structure"
```

---

## Task 2: Knowledge Entry Types and Frontmatter Parsing

**Files:**
- Modify: `src/knowledge/entry.rs`
- Create: `tests/entry_test.rs`

- [ ] **Step 1: Write the failing test for Entry parsing**

```rust
// tests/entry_test.rs
use mnemosyne::knowledge::entry::{Entry, Confidence, Origin};

#[test]
fn test_parse_entry_from_markdown() {
    let content = r#"---
title: Rust Async Patterns
tags: [rust, async, tokio]
created: 2026-03-31
last_validated: 2026-03-31
confidence: high
origins:
  - project: apianyware-macos
    date: 2026-03-31
    context: "Racket FFI async bridge work"
supersedes: []
---

## Bounded channels prevent backpressure bugs

**2026-03-31:** 🔴 Always use bounded channels in tokio.
"#;

    let entry = Entry::parse(content).unwrap();
    assert_eq!(entry.title, "Rust Async Patterns");
    assert_eq!(entry.tags, vec!["rust", "async", "tokio"]);
    assert_eq!(entry.confidence, Confidence::High);
    assert_eq!(entry.origins.len(), 1);
    assert_eq!(entry.origins[0].project, "apianyware-macos");
    assert!(entry.body.contains("Bounded channels"));
}

#[test]
fn test_parse_entry_with_prospective_confidence() {
    let content = r#"---
title: Error-Stack Crate
tags: [rust, error-handling]
created: 2026-04-01
last_validated: 2026-04-01
confidence: prospective
source: horizon-scan
origins:
  - project: global
    date: 2026-04-01
    context: "Discovered during exploration"
supersedes: []
---

## Assessment

Not yet evaluated in a real project.
"#;

    let entry = Entry::parse(content).unwrap();
    assert_eq!(entry.confidence, Confidence::Prospective);
    assert_eq!(entry.source.as_deref(), Some("horizon-scan"));
}

#[test]
fn test_parse_entry_missing_frontmatter_returns_error() {
    let content = "# No frontmatter here\n\nJust markdown.";
    assert!(Entry::parse(content).is_err());
}

#[test]
fn test_entry_serialize_roundtrip() {
    let content = r#"---
title: Test Entry
tags: [test, roundtrip]
created: 2026-03-31
last_validated: 2026-03-31
confidence: medium
origins:
  - project: test-project
    date: 2026-03-31
    context: "Testing serialization"
supersedes: []
---

## Some content

Body text here.
"#;

    let entry = Entry::parse(content).unwrap();
    let serialized = entry.serialize();
    let reparsed = Entry::parse(&serialized).unwrap();

    assert_eq!(entry.title, reparsed.title);
    assert_eq!(entry.tags, reparsed.tags);
    assert_eq!(entry.confidence, reparsed.confidence);
    assert!(reparsed.body.contains("Body text here."));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test entry_test`
Expected: Compilation error — `Entry`, `Confidence`, `Origin` not found

- [ ] **Step 3: Implement Entry types and parsing**

```rust
// src/knowledge/entry.rs
use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type Tag = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
    Prospective,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Origin {
    pub project: String,
    pub date: NaiveDate,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Frontmatter {
    pub title: String,
    pub tags: Vec<Tag>,
    pub created: NaiveDate,
    pub last_validated: NaiveDate,
    pub confidence: Confidence,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub origins: Vec<Origin>,
    #[serde(default)]
    pub supersedes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub title: String,
    pub tags: Vec<Tag>,
    pub created: NaiveDate,
    pub last_validated: NaiveDate,
    pub confidence: Confidence,
    pub source: Option<String>,
    pub origins: Vec<Origin>,
    pub supersedes: Vec<String>,
    pub body: String,
    pub file_path: Option<PathBuf>,
}

impl Entry {
    /// Parse a knowledge entry from markdown with YAML frontmatter.
    /// Expects the format: ---\n<yaml>\n---\n<markdown body>
    pub fn parse(content: &str) -> Result<Self> {
        let content = content.trim_start();
        if !content.starts_with("---") {
            return Err(anyhow!("Missing YAML frontmatter delimiter"));
        }

        let after_first = &content[3..];
        let end = after_first
            .find("\n---")
            .ok_or_else(|| anyhow!("Missing closing frontmatter delimiter"))?;

        let yaml_str = &after_first[..end];
        let body_start = end + 4; // skip \n---
        let body = after_first[body_start..].trim_start_matches('\n').to_string();

        let fm: Frontmatter =
            serde_yaml::from_str(yaml_str).context("Failed to parse frontmatter YAML")?;

        Ok(Self {
            title: fm.title,
            tags: fm.tags,
            created: fm.created,
            last_validated: fm.last_validated,
            confidence: fm.confidence,
            source: fm.source,
            origins: fm.origins,
            supersedes: fm.supersedes,
            body,
            file_path: None,
        })
    }

    /// Serialize the entry back to markdown with YAML frontmatter.
    pub fn serialize(&self) -> String {
        let fm = Frontmatter {
            title: self.title.clone(),
            tags: self.tags.clone(),
            created: self.created,
            last_validated: self.last_validated,
            confidence: self.confidence.clone(),
            source: self.source.clone(),
            origins: self.origins.clone(),
            supersedes: self.supersedes.clone(),
        };

        let yaml = serde_yaml::to_string(&fm).expect("Failed to serialize frontmatter");
        format!("---\n{}---\n\n{}\n", yaml, self.body.trim())
    }
}
```

- [ ] **Step 4: Ensure lib.rs exports correctly**

Update `src/knowledge/mod.rs`:

```rust
pub mod entry;
pub mod index;
pub mod store;
pub mod tags;
```

Verify `src/lib.rs` has:

```rust
pub mod config;
pub mod knowledge;
pub mod evolution;
pub mod context;
pub mod commands;
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --test entry_test`
Expected: All 4 tests pass

- [ ] **Step 6: Commit**

```bash
git add src/knowledge/entry.rs tests/entry_test.rs
git commit -m "feat: add Entry types with YAML frontmatter parsing and serialization"
```

---

## Task 3: Knowledge Store — Read/Write/Archive Entries

**Files:**
- Modify: `src/knowledge/store.rs`
- Create: `tests/store_test.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// tests/store_test.rs
use mnemosyne::knowledge::store::KnowledgeStore;
use mnemosyne::knowledge::entry::Confidence;
use std::fs;
use tempfile::TempDir;

fn create_test_entry(dir: &std::path::Path, subdir: &str, filename: &str, title: &str, tags: &[&str]) {
    let path = dir.join(subdir);
    fs::create_dir_all(&path).unwrap();
    let tag_str = tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");
    let content = format!(
        r#"---
title: {title}
tags: [{tag_str}]
created: 2026-03-31
last_validated: 2026-03-31
confidence: high
origins:
  - project: test
    date: 2026-03-31
    context: "test"
supersedes: []
---

## Content

Test content for {title}.
"#
    );
    fs::write(path.join(filename), content).unwrap();
}

#[test]
fn test_load_all_entries() {
    let tmp = TempDir::new().unwrap();
    let knowledge = tmp.path().join("knowledge");
    let archive = tmp.path().join("archive");
    fs::create_dir_all(&knowledge).unwrap();
    fs::create_dir_all(&archive).unwrap();

    create_test_entry(&knowledge, "languages", "rust.md", "Rust Patterns", &["rust"]);
    create_test_entry(&knowledge, "tools", "cargo.md", "Cargo Tips", &["cargo", "rust"]);

    let store = KnowledgeStore::new(knowledge, archive);
    let entries = store.load_all().unwrap();

    assert_eq!(entries.len(), 2);
    let titles: Vec<&str> = entries.iter().map(|e| e.title.as_str()).collect();
    assert!(titles.contains(&"Rust Patterns"));
    assert!(titles.contains(&"Cargo Tips"));
}

#[test]
fn test_load_all_skips_non_markdown_files() {
    let tmp = TempDir::new().unwrap();
    let knowledge = tmp.path().join("knowledge");
    let archive = tmp.path().join("archive");
    fs::create_dir_all(&knowledge).unwrap();
    fs::create_dir_all(&archive).unwrap();

    create_test_entry(&knowledge, "languages", "rust.md", "Rust", &["rust"]);
    fs::write(knowledge.join("README.txt"), "not a knowledge file").unwrap();

    let store = KnowledgeStore::new(knowledge, archive);
    let entries = store.load_all().unwrap();

    assert_eq!(entries.len(), 1);
}

#[test]
fn test_save_entry() {
    let tmp = TempDir::new().unwrap();
    let knowledge = tmp.path().join("knowledge");
    let archive = tmp.path().join("archive");
    fs::create_dir_all(&knowledge).unwrap();
    fs::create_dir_all(&archive).unwrap();

    create_test_entry(&knowledge, "languages", "rust.md", "Rust", &["rust"]);

    let store = KnowledgeStore::new(knowledge.clone(), archive);
    let mut entries = store.load_all().unwrap();
    let entry = &mut entries[0];
    entry.tags.push("systems".to_string());
    store.save_entry(entry).unwrap();

    // Re-load and verify
    let reloaded = store.load_all().unwrap();
    assert!(reloaded[0].tags.contains(&"systems".to_string()));
}

#[test]
fn test_archive_entry() {
    let tmp = TempDir::new().unwrap();
    let knowledge = tmp.path().join("knowledge");
    let archive = tmp.path().join("archive");
    fs::create_dir_all(&knowledge).unwrap();
    fs::create_dir_all(&archive).unwrap();

    create_test_entry(&knowledge, "languages", "rust.md", "Rust", &["rust"]);

    let store = KnowledgeStore::new(knowledge.clone(), archive.clone());
    let entries = store.load_all().unwrap();
    store.archive_entry(&entries[0], "No longer relevant").unwrap();

    // Original file should be gone
    assert!(!knowledge.join("languages/rust.md").exists());
    // Archive should have it
    let archived_files: Vec<_> = fs::read_dir(&archive)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!archived_files.is_empty());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test store_test`
Expected: Compilation error — `KnowledgeStore` not found

- [ ] **Step 3: Implement KnowledgeStore**

```rust
// src/knowledge/store.rs
use crate::knowledge::entry::Entry;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct KnowledgeStore {
    root: PathBuf,
    archive_root: PathBuf,
}

impl KnowledgeStore {
    pub fn new(root: PathBuf, archive_root: PathBuf) -> Self {
        Self { root, archive_root }
    }

    /// Load all knowledge entries from the knowledge directory tree.
    /// Walks all subdirectories, reads all .md files with valid frontmatter.
    pub fn load_all(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        self.walk_dir(&self.root, &mut entries)?;
        Ok(entries)
    }

    fn walk_dir(&self, dir: &Path, entries: &mut Vec<Entry>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for dir_entry in fs::read_dir(dir).context("Failed to read knowledge directory")? {
            let dir_entry = dir_entry?;
            let path = dir_entry.path();

            if path.is_dir() {
                self.walk_dir(&path, entries)?;
            } else if path.extension().is_some_and(|ext| ext == "md") {
                match self.load_entry(&path) {
                    Ok(entry) => entries.push(entry),
                    Err(_) => {
                        // Skip files that don't have valid frontmatter
                        // (e.g., CLAUDE.md, README.md)
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single entry from a file path.
    pub fn load_entry(&self, path: &Path) -> Result<Entry> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let mut entry = Entry::parse(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        entry.file_path = Some(path.to_path_buf());
        Ok(entry)
    }

    /// Save an entry back to its file. The entry must have a file_path set.
    pub fn save_entry(&self, entry: &Entry) -> Result<()> {
        let path = entry
            .file_path
            .as_ref()
            .context("Entry has no file_path — cannot save")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = entry.serialize();
        fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    /// Archive an entry: move it from knowledge/ to archive/ with a reason note.
    pub fn archive_entry(&self, entry: &Entry, reason: &str) -> Result<()> {
        let source_path = entry
            .file_path
            .as_ref()
            .context("Entry has no file_path — cannot archive")?;

        fs::create_dir_all(&self.archive_root)?;

        // Use timestamp + original filename to avoid collisions
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let filename = source_path
            .file_name()
            .context("Invalid file path")?
            .to_string_lossy();
        let archive_name = format!("{}-{}", timestamp, filename);
        let archive_path = self.archive_root.join(&archive_name);

        // Add archive reason to the entry body
        let mut archived = entry.clone();
        archived.body = format!(
            "{}\n\n## Archived\n\n**{}:** {}\n",
            entry.body.trim(),
            chrono::Local::now().format("%Y-%m-%d"),
            reason
        );
        archived.file_path = Some(archive_path);
        self.save_entry(&archived)?;

        // Remove the original
        fs::remove_file(source_path)
            .with_context(|| format!("Failed to remove {}", source_path.display()))?;

        Ok(())
    }

    /// Get the root path of the knowledge store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create a new entry file at the appropriate axis path.
    pub fn create_entry(&self, axis: &str, filename: &str, entry: &mut Entry) -> Result<()> {
        let dir = self.root.join(axis);
        fs::create_dir_all(&dir)?;
        let path = dir.join(filename);
        entry.file_path = Some(path);
        self.save_entry(entry)
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test store_test`
Expected: All 4 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/knowledge/store.rs tests/store_test.rs
git commit -m "feat: add KnowledgeStore for reading, writing, and archiving entries"
```

---

## Task 4: Tag System — Matching, Overlap, and Scoring

**Files:**
- Modify: `src/knowledge/tags.rs`
- Create: `tests/tags_test.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// tests/tags_test.rs
use mnemosyne::knowledge::tags::TagMatcher;

#[test]
fn test_overlap_count() {
    let a = vec!["rust".into(), "async".into(), "tokio".into()];
    let b = vec!["rust".into(), "tokio".into(), "networking".into()];
    assert_eq!(TagMatcher::overlap_count(&a, &b), 2);
}

#[test]
fn test_overlap_count_no_overlap() {
    let a = vec!["rust".into(), "async".into()];
    let b = vec!["python".into(), "django".into()];
    assert_eq!(TagMatcher::overlap_count(&a, &b), 0);
}

#[test]
fn test_overlap_score_normalized() {
    let a = vec!["rust".into(), "async".into(), "tokio".into()];
    let b = vec!["rust".into(), "async".into(), "tokio".into()];
    let score = TagMatcher::overlap_score(&a, &b);
    assert!((score - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_overlap_score_partial() {
    let a = vec!["rust".into(), "async".into(), "tokio".into(), "networking".into()];
    let b = vec!["rust".into(), "async".into()];
    // 2 overlapping out of 4 unique total = 0.5
    let score = TagMatcher::overlap_score(&a, &b);
    assert!(score > 0.0 && score < 1.0);
}

#[test]
fn test_overlap_score_empty() {
    let a: Vec<String> = vec![];
    let b: Vec<String> = vec![];
    assert!((TagMatcher::overlap_score(&a, &b)).abs() < f64::EPSILON);
}

#[test]
fn test_matches_any() {
    let entry_tags = vec!["rust".into(), "async".into(), "tokio".into()];
    let query_tags = vec!["python".into(), "tokio".into()];
    assert!(TagMatcher::matches_any(&entry_tags, &query_tags));
}

#[test]
fn test_matches_any_false() {
    let entry_tags = vec!["rust".into(), "async".into()];
    let query_tags = vec!["python".into(), "django".into()];
    assert!(!TagMatcher::matches_any(&entry_tags, &query_tags));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test tags_test`
Expected: Compilation error — `TagMatcher` not found

- [ ] **Step 3: Implement TagMatcher**

```rust
// src/knowledge/tags.rs
use std::collections::HashSet;

use crate::knowledge::entry::Tag;

pub struct TagMatcher;

impl TagMatcher {
    /// Count how many tags overlap between two tag lists.
    pub fn overlap_count(a: &[Tag], b: &[Tag]) -> usize {
        let set_a: HashSet<&str> = a.iter().map(|t| t.as_str()).collect();
        let set_b: HashSet<&str> = b.iter().map(|t| t.as_str()).collect();
        set_a.intersection(&set_b).count()
    }

    /// Compute a normalized overlap score between 0.0 and 1.0.
    /// Uses Jaccard similarity: |intersection| / |union|.
    pub fn overlap_score(a: &[Tag], b: &[Tag]) -> f64 {
        let set_a: HashSet<&str> = a.iter().map(|t| t.as_str()).collect();
        let set_b: HashSet<&str> = b.iter().map(|t| t.as_str()).collect();
        let union_size = set_a.union(&set_b).count();
        if union_size == 0 {
            return 0.0;
        }
        let intersection_size = set_a.intersection(&set_b).count();
        intersection_size as f64 / union_size as f64
    }

    /// Check if any tags from the query appear in the entry's tags.
    pub fn matches_any(entry_tags: &[Tag], query_tags: &[Tag]) -> bool {
        let entry_set: HashSet<&str> = entry_tags.iter().map(|t| t.as_str()).collect();
        query_tags.iter().any(|t| entry_set.contains(t.as_str()))
    }

    /// Check if all query tags appear in the entry's tags.
    pub fn matches_all(entry_tags: &[Tag], query_tags: &[Tag]) -> bool {
        let entry_set: HashSet<&str> = entry_tags.iter().map(|t| t.as_str()).collect();
        query_tags.iter().all(|t| entry_set.contains(t.as_str()))
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test tags_test`
Expected: All 7 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/knowledge/tags.rs tests/tags_test.rs
git commit -m "feat: add TagMatcher with overlap scoring and matching"
```

---

## Task 5: KnowledgeIndex Trait and FileIndex Implementation

**Files:**
- Modify: `src/knowledge/index.rs`
- Create: `tests/index_test.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// tests/index_test.rs
use mnemosyne::knowledge::entry::{Confidence, Entry, Origin};
use mnemosyne::knowledge::index::{FileIndex, KnowledgeIndex, Query};
use chrono::NaiveDate;

fn make_entry(title: &str, tags: &[&str], confidence: Confidence) -> Entry {
    Entry {
        title: title.to_string(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        created: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        last_validated: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        confidence,
        source: None,
        origins: vec![Origin {
            project: "test".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            context: "test".to_string(),
        }],
        supersedes: vec![],
        body: format!("Content about {}", title),
        file_path: None,
    }
}

#[test]
fn test_search_by_text() {
    let entries = vec![
        make_entry("Rust Async", &["rust", "async"], Confidence::High),
        make_entry("Python Django", &["python", "django"], Confidence::High),
    ];
    let index = FileIndex::from_entries(entries);

    let results = index.search(&Query {
        text: Some("Rust".to_string()),
        tags: vec![],
    });

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.title, "Rust Async");
}

#[test]
fn test_search_by_tags() {
    let entries = vec![
        make_entry("Rust Async", &["rust", "async"], Confidence::High),
        make_entry("Python Async", &["python", "async"], Confidence::High),
        make_entry("Rust Types", &["rust", "type-system"], Confidence::High),
    ];
    let index = FileIndex::from_entries(entries);

    let results = index.search(&Query {
        text: None,
        tags: vec!["async".to_string()],
    });

    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_results_ordered_by_relevance() {
    let entries = vec![
        make_entry("Broad", &["rust"], Confidence::Low),
        make_entry("Specific", &["rust", "async", "tokio"], Confidence::High),
    ];
    let index = FileIndex::from_entries(entries);

    let results = index.search(&Query {
        text: None,
        tags: vec!["rust".to_string(), "async".to_string(), "tokio".to_string()],
    });

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].entry.title, "Specific");
}

#[test]
fn test_find_by_tags() {
    let entries = vec![
        make_entry("Rust Async", &["rust", "async"], Confidence::High),
        make_entry("Python", &["python"], Confidence::High),
    ];
    let index = FileIndex::from_entries(entries);

    let found = index.find_by_tags(&["rust".to_string()]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].title, "Rust Async");
}

#[test]
fn test_find_contradictions_by_tag_overlap() {
    let entries = vec![
        make_entry("Use unbounded channels", &["rust", "async", "channels"], Confidence::High),
        make_entry("Other topic", &["python"], Confidence::High),
    ];
    let index = FileIndex::from_entries(entries);

    let new_entry = make_entry("Use bounded channels", &["rust", "async", "channels"], Confidence::High);
    let contradictions = index.find_contradictions(&new_entry);

    assert_eq!(contradictions.len(), 1);
    assert_eq!(contradictions[0].existing.title, "Use unbounded channels");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test index_test`
Expected: Compilation error — `FileIndex`, `KnowledgeIndex`, `Query` not found

- [ ] **Step 3: Implement KnowledgeIndex trait and FileIndex**

```rust
// src/knowledge/index.rs
use crate::knowledge::entry::{Confidence, Entry, Tag};
use crate::knowledge::tags::TagMatcher;

/// A query against the knowledge index.
pub struct Query {
    pub text: Option<String>,
    pub tags: Vec<Tag>,
}

/// A single search result with a relevance score.
pub struct SearchResult {
    pub entry: Entry,
    pub score: f64,
}

/// A potential contradiction between a new entry and an existing one.
pub struct PotentialContradiction {
    pub existing: Entry,
    pub overlap_score: f64,
}

/// Trait for knowledge index implementations.
/// v1: FileIndex (in-memory). v2: could add VectorIndex.
pub trait KnowledgeIndex {
    fn search(&self, query: &Query) -> Vec<SearchResult>;
    fn find_contradictions(&self, entry: &Entry) -> Vec<PotentialContradiction>;
    fn find_by_tags(&self, tags: &[Tag]) -> Vec<&Entry>;
}

/// In-memory index built from scanning knowledge files.
pub struct FileIndex {
    entries: Vec<Entry>,
}

impl FileIndex {
    pub fn from_entries(entries: Vec<Entry>) -> Self {
        Self { entries }
    }

    /// Score an entry against a query.
    fn score_entry(entry: &Entry, query: &Query) -> f64 {
        let mut score = 0.0;

        // Tag overlap scoring
        if !query.tags.is_empty() {
            let tag_score = TagMatcher::overlap_score(&entry.tags, &query.tags);
            score += tag_score * 10.0;
        }

        // Text matching in title and body
        if let Some(ref text) = query.text {
            let text_lower = text.to_lowercase();
            if entry.title.to_lowercase().contains(&text_lower) {
                score += 5.0;
            }
            if entry.body.to_lowercase().contains(&text_lower) {
                score += 2.0;
            }
        }

        // Confidence bonus
        score *= match entry.confidence {
            Confidence::High => 1.0,
            Confidence::Medium => 0.8,
            Confidence::Low => 0.6,
            Confidence::Prospective => 0.4,
        };

        score
    }
}

impl KnowledgeIndex for FileIndex {
    fn search(&self, query: &Query) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let score = Self::score_entry(entry, query);
                if score > 0.0 {
                    Some(SearchResult {
                        entry: entry.clone(),
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    fn find_contradictions(&self, entry: &Entry) -> Vec<PotentialContradiction> {
        let threshold = 0.5; // Minimum Jaccard overlap to flag as potential contradiction

        let mut contradictions: Vec<PotentialContradiction> = self
            .entries
            .iter()
            .filter_map(|existing| {
                let overlap = TagMatcher::overlap_score(&existing.tags, &entry.tags);
                if overlap >= threshold {
                    Some(Contradiction {
                        existing: existing.clone(),
                        overlap_score: overlap,
                    })
                } else {
                    None
                }
            })
            .collect();

        contradictions.sort_by(|a, b| {
            b.overlap_score
                .partial_cmp(&a.overlap_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        contradictions
    }

    fn find_by_tags(&self, tags: &[Tag]) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|entry| TagMatcher::matches_any(&entry.tags, tags))
            .collect()
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test index_test`
Expected: All 5 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/knowledge/index.rs tests/index_test.rs
git commit -m "feat: add KnowledgeIndex trait with FileIndex implementation"
```

---

## Task 6: Configuration Parsing

**Files:**
- Modify: `src/config.rs`
- Create: `tests/config_test.rs`
- Create: `defaults/config.yml`

- [ ] **Step 1: Write the failing tests**

```rust
// tests/config_test.rs
use mnemosyne::config::Config;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_parse_config() {
    let yaml = r#"
language_profiles:
  rust:
    markers: ["Cargo.toml"]
    extensions: [".rs"]
    dependency_file: "Cargo.toml"
    dependency_parser: cargo
  python:
    markers: ["pyproject.toml", "setup.py"]
    extensions: [".py"]
context_mappings:
  cargo_dependencies:
    tokio: [async, tokio, concurrency]
    sqlx: [database, sql]
"#;

    let config = Config::parse(yaml).unwrap();
    assert!(config.language_profiles.contains_key("rust"));
    assert_eq!(config.language_profiles["rust"].markers, vec!["Cargo.toml"]);
    assert_eq!(config.language_profiles["rust"].extensions, vec![".rs"]);
    assert_eq!(
        config.language_profiles["rust"].dependency_parser.as_deref(),
        Some("cargo")
    );
    assert!(config.context_mappings.contains_key("cargo_dependencies"));
}

#[test]
fn test_load_config_from_file() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.yml");
    fs::write(
        &config_path,
        "language_profiles:\n  rust:\n    markers: [\"Cargo.toml\"]\n    extensions: [\".rs\"]\n",
    )
    .unwrap();

    let config = Config::load(tmp.path()).unwrap();
    assert!(config.language_profiles.contains_key("rust"));
}

#[test]
fn test_load_config_returns_defaults_when_missing() {
    let tmp = TempDir::new().unwrap();
    let config = Config::load(tmp.path()).unwrap();
    // Should have default language profiles
    assert!(!config.language_profiles.is_empty());
    assert!(config.language_profiles.contains_key("rust"));
}

#[test]
fn test_default_config_has_all_expected_languages() {
    let config = Config::default();
    let expected = [
        "rust", "python", "haskell", "ocaml", "prolog", "mercury",
        "scheme", "racket", "common-lisp", "smalltalk", "idris", "swift",
    ];
    for lang in expected {
        assert!(
            config.language_profiles.contains_key(lang),
            "Missing language profile: {}",
            lang
        );
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test config_test`
Expected: Compilation error — `Config` not found

- [ ] **Step 3: Implement Config**

```rust
// src/config.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageProfile {
    #[serde(default)]
    pub markers: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub dependency_file: Option<String>,
    #[serde(default)]
    pub dependency_parser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub language_profiles: HashMap<String, LanguageProfile>,
    #[serde(default)]
    pub context_mappings: HashMap<String, HashMap<String, Vec<String>>>,
}

impl Config {
    /// Parse config from a YAML string.
    pub fn parse(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse config YAML")
    }

    /// Load config from a directory. Reads config.yml if present, otherwise returns defaults.
    pub fn load(dir: &Path) -> Result<Self> {
        let config_path = dir.join("config.yml");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read {}", config_path.display()))?;
            Self::parse(&content)
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to a directory.
    pub fn save(&self, dir: &Path) -> Result<()> {
        let config_path = dir.join("config.yml");
        let yaml = serde_yaml::to_string(self).context("Failed to serialize config")?;
        fs::write(&config_path, yaml)
            .with_context(|| format!("Failed to write {}", config_path.display()))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language_profiles: default_language_profiles(),
            context_mappings: default_context_mappings(),
        }
    }
}

fn default_language_profiles() -> HashMap<String, LanguageProfile> {
    let mut profiles = HashMap::new();

    profiles.insert("rust".into(), LanguageProfile {
        markers: vec!["Cargo.toml".into()],
        extensions: vec![".rs".into()],
        dependency_file: Some("Cargo.toml".into()),
        dependency_parser: Some("cargo".into()),
    });
    profiles.insert("python".into(), LanguageProfile {
        markers: vec!["pyproject.toml".into(), "setup.py".into(), "requirements.txt".into()],
        extensions: vec![".py".into()],
        dependency_file: Some("pyproject.toml".into()),
        dependency_parser: Some("pyproject".into()),
    });
    profiles.insert("haskell".into(), LanguageProfile {
        markers: vec!["*.cabal".into(), "stack.yaml".into(), "cabal.project".into()],
        extensions: vec![".hs".into()],
        dependency_file: None,
        dependency_parser: Some("cabal".into()),
    });
    profiles.insert("ocaml".into(), LanguageProfile {
        markers: vec!["dune-project".into(), "*.opam".into()],
        extensions: vec![".ml".into(), ".mli".into()],
        dependency_file: None,
        dependency_parser: Some("opam".into()),
    });
    profiles.insert("prolog".into(), LanguageProfile {
        markers: vec!["pack.pl".into()],
        extensions: vec![".pl".into(), ".pro".into()],
        dependency_file: None,
        dependency_parser: None,
    });
    profiles.insert("mercury".into(), LanguageProfile {
        markers: vec!["Mercury.options".into()],
        extensions: vec![".m".into(), ".mh".into()],
        dependency_file: None,
        dependency_parser: None,
    });
    profiles.insert("scheme".into(), LanguageProfile {
        markers: vec![],
        extensions: vec![".scm".into(), ".ss".into(), ".sld".into()],
        dependency_file: None,
        dependency_parser: None,
    });
    profiles.insert("racket".into(), LanguageProfile {
        markers: vec!["info.rkt".into()],
        extensions: vec![".rkt".into()],
        dependency_file: None,
        dependency_parser: None,
    });
    profiles.insert("common-lisp".into(), LanguageProfile {
        markers: vec!["*.asd".into()],
        extensions: vec![".lisp".into(), ".cl".into(), ".lsp".into()],
        dependency_file: None,
        dependency_parser: None,
    });
    profiles.insert("smalltalk".into(), LanguageProfile {
        markers: vec![".smalltalk.ston".into()],
        extensions: vec![".st".into()],
        dependency_file: None,
        dependency_parser: None,
    });
    profiles.insert("idris".into(), LanguageProfile {
        markers: vec!["*.ipkg".into()],
        extensions: vec![".idr".into()],
        dependency_file: None,
        dependency_parser: None,
    });
    profiles.insert("swift".into(), LanguageProfile {
        markers: vec!["Package.swift".into()],
        extensions: vec![".swift".into()],
        dependency_file: Some("Package.swift".into()),
        dependency_parser: None,
    });

    profiles
}

fn default_context_mappings() -> HashMap<String, HashMap<String, Vec<String>>> {
    let mut mappings = HashMap::new();

    let mut cargo = HashMap::new();
    cargo.insert("tokio".into(), vec!["async".into(), "tokio".into(), "concurrency".into()]);
    cargo.insert("sqlx".into(), vec!["database".into(), "sql".into(), "async".into()]);
    cargo.insert("axum".into(), vec!["web".into(), "http".into(), "api".into()]);
    cargo.insert("serde".into(), vec!["serialization".into(), "serde".into()]);
    mappings.insert("cargo_dependencies".into(), cargo);

    mappings
}
```

- [ ] **Step 4: Create defaults/config.yml**

```yaml
# Mnemosyne default configuration
# This file is generated by `mnemosyne init` and can be customized.

# Language detection profiles.
# Each profile defines how to detect a language in a project directory.
# markers: files whose presence signals this language
# extensions: file extensions for this language
# dependency_file: which file lists dependencies
# dependency_parser: parser name for extracting dependency info
language_profiles:
  rust:
    markers: ["Cargo.toml"]
    extensions: [".rs"]
    dependency_file: "Cargo.toml"
    dependency_parser: cargo
  python:
    markers: ["pyproject.toml", "setup.py", "requirements.txt"]
    extensions: [".py"]
    dependency_file: "pyproject.toml"
    dependency_parser: pyproject
  haskell:
    markers: ["*.cabal", "stack.yaml", "cabal.project"]
    extensions: [".hs"]
    dependency_parser: cabal
  ocaml:
    markers: ["dune-project", "*.opam"]
    extensions: [".ml", ".mli"]
    dependency_parser: opam
  prolog:
    markers: ["pack.pl"]
    extensions: [".pl", ".pro"]
  mercury:
    markers: ["Mercury.options"]
    extensions: [".m", ".mh"]
  scheme:
    extensions: [".scm", ".ss", ".sld"]
  racket:
    markers: ["info.rkt"]
    extensions: [".rkt"]
  common-lisp:
    markers: ["*.asd"]
    extensions: [".lisp", ".cl", ".lsp"]
  smalltalk:
    markers: [".smalltalk.ston"]
    extensions: [".st"]
  idris:
    markers: ["*.ipkg"]
    extensions: [".idr"]
  swift:
    markers: ["Package.swift"]
    extensions: [".swift"]

# Context mappings.
# Maps dependency names to knowledge tags for context-inferred retrieval.
context_mappings:
  cargo_dependencies:
    tokio: [async, tokio, concurrency]
    sqlx: [database, sql, async]
    axum: [web, http, api]
    serde: [serialization, serde]
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --test config_test`
Expected: All 4 tests pass

- [ ] **Step 6: Commit**

```bash
git add src/config.rs tests/config_test.rs defaults/
git commit -m "feat: add Config with language profiles and context mappings"
```

---

## Task 7: Project Signal Detection

**Files:**
- Modify: `src/context/detect.rs`
- Modify: `src/context/mapping.rs`
- Create: `tests/detect_test.rs`
- Create: `tests/mapping_test.rs`

- [ ] **Step 1: Write detect tests**

```rust
// tests/detect_test.rs
use mnemosyne::config::Config;
use mnemosyne::context::detect::{ProjectDetector, Signal};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_detect_rust_project() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\n[dependencies]\ntokio = \"1\"\n",
    )
    .unwrap();
    fs::create_dir(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.rs"), "fn main() {}").unwrap();

    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let signals = detector.detect(tmp.path()).unwrap();

    assert!(signals.iter().any(|s| matches!(s, Signal::Language(lang) if lang == "rust")));
}

#[test]
fn test_detect_python_project() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();
    fs::write(tmp.path().join("main.py"), "print('hello')").unwrap();

    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let signals = detector.detect(tmp.path()).unwrap();

    assert!(signals.iter().any(|s| matches!(s, Signal::Language(lang) if lang == "python")));
}

#[test]
fn test_detect_project_name_from_git() {
    let tmp = TempDir::new().unwrap();
    let git_dir = tmp.path().join(".git");
    fs::create_dir(&git_dir).unwrap();
    fs::write(
        git_dir.join("config"),
        "[remote \"origin\"]\n\turl = git@github.com:user/my-project.git\n",
    )
    .unwrap();

    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let signals = detector.detect(tmp.path()).unwrap();

    assert!(signals.iter().any(|s| matches!(s, Signal::ProjectName(name) if name == "my-project")));
}

#[test]
fn test_detect_multiple_languages() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    fs::write(tmp.path().join("Package.swift"), "// swift-tools-version:5.5\n").unwrap();

    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let signals = detector.detect(tmp.path()).unwrap();

    let languages: Vec<&str> = signals
        .iter()
        .filter_map(|s| match s {
            Signal::Language(l) => Some(l.as_str()),
            _ => None,
        })
        .collect();
    assert!(languages.contains(&"rust"));
    assert!(languages.contains(&"swift"));
}
```

- [ ] **Step 2: Write mapping tests**

```rust
// tests/mapping_test.rs
use mnemosyne::config::Config;
use mnemosyne::context::detect::Signal;
use mnemosyne::context::mapping::SignalMapper;

#[test]
fn test_map_language_signal_to_tags() {
    let config = Config::default();
    let mapper = SignalMapper::new(&config);

    let signals = vec![Signal::Language("rust".to_string())];
    let tags = mapper.map_signals(&signals);

    assert!(tags.contains(&"rust".to_string()));
}

#[test]
fn test_map_dependency_signal_to_tags() {
    let config = Config::default();
    let mapper = SignalMapper::new(&config);

    let signals = vec![
        Signal::Language("rust".to_string()),
        Signal::Dependency {
            ecosystem: "cargo_dependencies".to_string(),
            name: "tokio".to_string(),
        },
    ];
    let tags = mapper.map_signals(&signals);

    assert!(tags.contains(&"rust".to_string()));
    assert!(tags.contains(&"async".to_string()));
    assert!(tags.contains(&"tokio".to_string()));
}

#[test]
fn test_map_unknown_dependency_uses_name_as_tag() {
    let config = Config::default();
    let mapper = SignalMapper::new(&config);

    let signals = vec![Signal::Dependency {
        ecosystem: "cargo_dependencies".to_string(),
        name: "obscure-crate".to_string(),
    }];
    let tags = mapper.map_signals(&signals);

    assert!(tags.contains(&"obscure-crate".to_string()));
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --test detect_test --test mapping_test`
Expected: Compilation errors

- [ ] **Step 4: Implement ProjectDetector**

```rust
// src/context/detect.rs
use crate::config::Config;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// A signal detected from a project directory.
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    /// A programming language detected in the project.
    Language(String),
    /// A dependency detected from a manifest file.
    Dependency { ecosystem: String, name: String },
    /// The project name (from git remote or directory name).
    ProjectName(String),
}

pub struct ProjectDetector<'a> {
    config: &'a Config,
}

impl<'a> ProjectDetector<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Detect signals from a project directory.
    pub fn detect(&self, project_root: &Path) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();

        self.detect_languages(project_root, &mut signals)?;
        self.detect_project_name(project_root, &mut signals)?;

        Ok(signals)
    }

    fn detect_languages(&self, root: &Path, signals: &mut Vec<Signal>) -> Result<()> {
        for (lang_name, profile) in &self.config.language_profiles {
            let detected = profile.markers.iter().any(|marker| {
                if marker.contains('*') {
                    // Glob pattern — check if any file matches
                    self.glob_matches(root, marker)
                } else {
                    root.join(marker).exists()
                }
            });

            if !detected {
                // Fall back to extension detection
                let has_extension = profile.extensions.iter().any(|ext| {
                    self.has_files_with_extension(root, ext)
                });
                if !has_extension {
                    continue;
                }
            }

            signals.push(Signal::Language(lang_name.clone()));

            // Try to extract dependencies
            if let Some(ref dep_file) = profile.dependency_file {
                if let Some(ref parser) = profile.dependency_parser {
                    let dep_path = root.join(dep_file);
                    if dep_path.exists() {
                        self.extract_dependencies(&dep_path, parser, signals)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn glob_matches(&self, root: &Path, pattern: &str) -> bool {
        let Ok(entries) = fs::read_dir(root) else {
            return false;
        };

        let suffix = pattern.trim_start_matches('*');
        entries.filter_map(|e| e.ok()).any(|e| {
            e.file_name().to_string_lossy().ends_with(suffix)
        })
    }

    fn has_files_with_extension(&self, root: &Path, ext: &str) -> bool {
        let Ok(entries) = fs::read_dir(root) else {
            return false;
        };

        entries.filter_map(|e| e.ok()).any(|e| {
            e.file_name().to_string_lossy().ends_with(ext)
        })
    }

    fn extract_dependencies(
        &self,
        dep_path: &Path,
        parser: &str,
        signals: &mut Vec<Signal>,
    ) -> Result<()> {
        let content = fs::read_to_string(dep_path)?;
        let ecosystem = format!("{}_dependencies", parser);

        match parser {
            "cargo" => self.parse_cargo_deps(&content, &ecosystem, signals),
            "pyproject" => self.parse_pyproject_deps(&content, &ecosystem, signals),
            _ => {} // Unknown parser — skip dependency extraction
        }

        Ok(())
    }

    fn parse_cargo_deps(&self, content: &str, ecosystem: &str, signals: &mut Vec<Signal>) {
        // Simple parser: look for lines under [dependencies] that start with a crate name
        let mut in_deps = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[dependencies]" || trimmed == "[dev-dependencies]" {
                in_deps = true;
                continue;
            }
            if trimmed.starts_with('[') {
                in_deps = false;
                continue;
            }
            if in_deps {
                if let Some(name) = trimmed.split('=').next() {
                    let name = name.trim();
                    if !name.is_empty() && !name.starts_with('#') {
                        signals.push(Signal::Dependency {
                            ecosystem: ecosystem.to_string(),
                            name: name.to_string(),
                        });
                    }
                }
            }
        }
    }

    fn parse_pyproject_deps(&self, content: &str, ecosystem: &str, signals: &mut Vec<Signal>) {
        // Simple parser: look for lines under [project.dependencies] or dependencies = [...]
        let mut in_deps = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "dependencies = [" || trimmed == "[project.dependencies]" {
                in_deps = true;
                continue;
            }
            if in_deps && trimmed == "]" {
                in_deps = false;
                continue;
            }
            if in_deps {
                // Extract package name from "package>=version" or "package"
                let clean = trimmed.trim_matches(|c: char| c == '"' || c == '\'' || c == ',');
                if let Some(name) = clean.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_').next() {
                    let name = name.trim();
                    if !name.is_empty() {
                        signals.push(Signal::Dependency {
                            ecosystem: ecosystem.to_string(),
                            name: name.to_string(),
                        });
                    }
                }
            }
        }
    }

    fn detect_project_name(&self, root: &Path, signals: &mut Vec<Signal>) -> Result<()> {
        let git_config = root.join(".git/config");
        if git_config.exists() {
            let content = fs::read_to_string(&git_config)?;
            if let Some(name) = self.extract_project_name_from_git(&content) {
                signals.push(Signal::ProjectName(name));
                return Ok(());
            }
        }

        // Fall back to directory name
        if let Some(dir_name) = root.file_name() {
            signals.push(Signal::ProjectName(dir_name.to_string_lossy().to_string()));
        }

        Ok(())
    }

    fn extract_project_name_from_git(&self, git_config: &str) -> Option<String> {
        for line in git_config.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("url = ") {
                let url = trimmed.strip_prefix("url = ")?;
                // Extract repo name from URL
                let name = url
                    .rsplit('/')
                    .next()?
                    .strip_suffix(".git")
                    .unwrap_or(url.rsplit('/').next()?);
                return Some(name.to_string());
            }
        }
        None
    }
}
```

- [ ] **Step 5: Implement SignalMapper**

```rust
// src/context/mapping.rs
use crate::config::Config;
use crate::context::detect::Signal;
use crate::knowledge::entry::Tag;
use std::collections::HashSet;

pub struct SignalMapper<'a> {
    config: &'a Config,
}

impl<'a> SignalMapper<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Convert a list of project signals into knowledge tags.
    pub fn map_signals(&self, signals: &[Signal]) -> Vec<Tag> {
        let mut tags = HashSet::new();

        for signal in signals {
            match signal {
                Signal::Language(lang) => {
                    tags.insert(lang.clone());
                }
                Signal::Dependency { ecosystem, name } => {
                    // Check if we have a mapping for this dependency
                    if let Some(eco_mappings) = self.config.context_mappings.get(ecosystem) {
                        if let Some(mapped_tags) = eco_mappings.get(name) {
                            for tag in mapped_tags {
                                tags.insert(tag.clone());
                            }
                        } else {
                            // No mapping — use the dependency name itself as a tag
                            tags.insert(name.clone());
                        }
                    } else {
                        tags.insert(name.clone());
                    }
                }
                Signal::ProjectName(_) => {
                    // Project names don't map to knowledge tags
                }
            }
        }

        let mut result: Vec<Tag> = tags.into_iter().collect();
        result.sort();
        result
    }
}
```

- [ ] **Step 6: Run the tests**

Run: `cargo test --test detect_test --test mapping_test`
Expected: All 6 tests pass

- [ ] **Step 7: Commit**

```bash
git add src/context/detect.rs src/context/mapping.rs tests/detect_test.rs tests/mapping_test.rs
git commit -m "feat: add project signal detection and tag mapping"
```

---

## Task 8: Evolution — Contradiction Detection and Supersession

**Files:**
- Modify: `src/evolution/contradiction.rs`
- Modify: `src/evolution/supersede.rs`
- Modify: `src/evolution/divergence.rs`
- Create: `tests/contradiction_test.rs`
- Create: `tests/supersede_test.rs`
- Create: `tests/divergence_test.rs`

- [ ] **Step 1: Write contradiction tests**

```rust
// tests/contradiction_test.rs
use chrono::NaiveDate;
use mnemosyne::evolution::contradiction::ContradictionDetector;
use mnemosyne::knowledge::entry::{Confidence, Entry, Origin};

fn make_entry(title: &str, tags: &[&str], project: &str) -> Entry {
    Entry {
        title: title.to_string(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        created: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        last_validated: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        confidence: Confidence::High,
        source: None,
        origins: vec![Origin {
            project: project.to_string(),
            date: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            context: "test".to_string(),
        }],
        supersedes: vec![],
        body: format!("Content about {}", title),
        file_path: None,
    }
}

#[test]
fn test_detect_contradiction_high_overlap() {
    let existing = vec![
        make_entry("Use unbounded channels", &["rust", "async", "channels"], "project-a"),
    ];
    let new_entry = make_entry("Use bounded channels", &["rust", "async", "channels"], "project-b");

    let detector = ContradictionDetector::new(0.5);
    let results = detector.detect(&existing, &new_entry);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].existing.title, "Use unbounded channels");
}

#[test]
fn test_no_contradiction_low_overlap() {
    let existing = vec![
        make_entry("Rust patterns", &["rust", "patterns"], "project-a"),
    ];
    let new_entry = make_entry("Python patterns", &["python", "patterns"], "project-b");

    let detector = ContradictionDetector::new(0.5);
    let results = detector.detect(&existing, &new_entry);

    assert!(results.is_empty());
}
```

- [ ] **Step 2: Write supersede tests**

```rust
// tests/supersede_test.rs
use chrono::NaiveDate;
use mnemosyne::evolution::supersede::supersede_content;
use mnemosyne::knowledge::entry::{Confidence, Entry, Origin};

#[test]
fn test_supersede_adds_section() {
    let mut entry = Entry {
        title: "Channel Patterns".to_string(),
        tags: vec!["rust".into(), "channels".into()],
        created: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        last_validated: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        confidence: Confidence::High,
        source: None,
        origins: vec![Origin {
            project: "project-a".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            context: "initial work".to_string(),
        }],
        supersedes: vec![],
        body: "## Use unbounded channels\n\n**2026-01-15:** Prefer unbounded for logging.".to_string(),
        file_path: None,
    };

    let old_content = "Use unbounded channels";
    let reason = "Caused memory exhaustion under sustained load";

    supersede_content(
        &mut entry,
        old_content,
        reason,
        NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
    );

    assert!(entry.body.contains("## Superseded"));
    assert!(entry.body.contains("2026-01-15 → 2026-03-31"));
    assert!(entry.body.contains(reason));
    assert!(entry.body.contains(old_content));
}
```

- [ ] **Step 3: Write divergence tests**

```rust
// tests/divergence_test.rs
use chrono::NaiveDate;
use mnemosyne::evolution::divergence::DivergenceDetector;
use mnemosyne::knowledge::entry::{Confidence, Entry, Origin};

fn make_entry_with_origins(title: &str, tags: &[&str], origins: Vec<(&str, &str)>) -> Entry {
    Entry {
        title: title.to_string(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        created: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        last_validated: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        confidence: Confidence::High,
        source: None,
        origins: origins
            .into_iter()
            .map(|(project, date_str)| {
                let parts: Vec<u32> = date_str.split('-').map(|p| p.parse().unwrap()).collect();
                Origin {
                    project: project.to_string(),
                    date: NaiveDate::from_ymd_opt(parts[0] as i32, parts[1], parts[2]).unwrap(),
                    context: "test".to_string(),
                }
            })
            .collect(),
        supersedes: vec![],
        body: format!("Content about {}", title),
        file_path: None,
    }
}

#[test]
fn test_detect_divergence_multiple_projects() {
    let global = vec![
        make_entry_with_origins(
            "Prefer integration tests",
            &["testing", "database"],
            vec![("project-a", "2026-01-01")],
        ),
    ];

    // Recent entries from different projects that diverge
    let recent = vec![
        make_entry_with_origins(
            "Unit tests with containers",
            &["testing", "database"],
            vec![("project-b", "2026-03-15")],
        ),
        make_entry_with_origins(
            "Test containers approach",
            &["testing", "database"],
            vec![("project-c", "2026-03-20")],
        ),
    ];

    let detector = DivergenceDetector::new(0.5, 2);
    let flagged = detector.detect(&global, &recent);

    assert_eq!(flagged.len(), 1);
    assert_eq!(flagged[0].entry.title, "Prefer integration tests");
    assert_eq!(flagged[0].diverging_count, 2);
}

#[test]
fn test_no_divergence_single_project() {
    let global = vec![
        make_entry_with_origins(
            "Prefer integration tests",
            &["testing", "database"],
            vec![("project-a", "2026-01-01")],
        ),
    ];

    let recent = vec![
        make_entry_with_origins(
            "Unit tests approach",
            &["testing", "database"],
            vec![("project-b", "2026-03-15")],
        ),
    ];

    let detector = DivergenceDetector::new(0.5, 2);
    let flagged = detector.detect(&global, &recent);

    assert!(flagged.is_empty()); // Only 1 diverging project, threshold is 2
}
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cargo test --test contradiction_test --test supersede_test --test divergence_test`
Expected: Compilation errors

- [ ] **Step 5: Implement ContradictionDetector**

```rust
// src/evolution/contradiction.rs
use crate::knowledge::entry::Entry;
use crate::knowledge::tags::TagMatcher;

/// A potential contradiction between a new entry and an existing one.
pub struct PotentialContradiction {
    pub existing: Entry,
    pub overlap_score: f64,
}

pub struct ContradictionDetector {
    threshold: f64,
}

impl ContradictionDetector {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Find existing entries that potentially contradict a new entry.
    /// Uses tag overlap as the primary signal.
    pub fn detect(&self, existing: &[Entry], new_entry: &Entry) -> Vec<PotentialContradiction> {
        let mut results: Vec<PotentialContradiction> = existing
            .iter()
            .filter_map(|entry| {
                let score = TagMatcher::overlap_score(&entry.tags, &new_entry.tags);
                if score >= self.threshold {
                    Some(PotentialContradiction {
                        existing: entry.clone(),
                        overlap_score: score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.overlap_score
                .partial_cmp(&a.overlap_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }
}
```

- [ ] **Step 6: Implement supersede_content**

```rust
// src/evolution/supersede.rs
use crate::knowledge::entry::Entry;
use chrono::NaiveDate;

/// Add superseded content to an entry's body.
/// Moves the specified old content into a ## Superseded section with a date range and reason.
pub fn supersede_content(
    entry: &mut Entry,
    old_content_summary: &str,
    reason: &str,
    supersede_date: NaiveDate,
) {
    let created_date = entry.created;
    let superseded_section = format!(
        "\n\n## Superseded\n\n### {} ({} → {})\n> {}\n\n**Reason superseded:** {}\n",
        old_content_summary, created_date, supersede_date, old_content_summary, reason
    );

    if entry.body.contains("## Superseded") {
        // Append to existing Superseded section
        entry.body.push_str(&format!(
            "\n### {} ({} → {})\n> {}\n\n**Reason superseded:** {}\n",
            old_content_summary, created_date, supersede_date, old_content_summary, reason
        ));
    } else {
        entry.body.push_str(&superseded_section);
    }
}
```

- [ ] **Step 7: Implement DivergenceDetector**

```rust
// src/evolution/divergence.rs
use crate::knowledge::entry::Entry;
use crate::knowledge::tags::TagMatcher;
use std::collections::HashSet;

/// A global entry flagged for potential divergence.
pub struct DivergenceFlag {
    pub entry: Entry,
    pub diverging_count: usize,
    pub diverging_projects: Vec<String>,
}

pub struct DivergenceDetector {
    tag_threshold: f64,
    project_threshold: usize,
}

impl DivergenceDetector {
    /// Create a new divergence detector.
    /// - `tag_threshold`: minimum tag overlap to consider entries related
    /// - `project_threshold`: minimum number of distinct projects that must diverge
    pub fn new(tag_threshold: f64, project_threshold: usize) -> Self {
        Self {
            tag_threshold,
            project_threshold,
        }
    }

    /// Detect global entries that may be diverged from by recent project-local entries.
    /// `global` — existing global knowledge entries
    /// `recent` — recently promoted entries from various projects
    pub fn detect(&self, global: &[Entry], recent: &[Entry]) -> Vec<DivergenceFlag> {
        let mut flags = Vec::new();

        for global_entry in global {
            let mut diverging_projects = HashSet::new();

            for recent_entry in recent {
                let overlap =
                    TagMatcher::overlap_score(&global_entry.tags, &recent_entry.tags);
                if overlap >= self.tag_threshold {
                    // This recent entry overlaps with the global entry.
                    // Consider it a divergence signal from each of its origin projects.
                    for origin in &recent_entry.origins {
                        // Don't count the same project that created the global entry
                        let global_projects: HashSet<&str> = global_entry
                            .origins
                            .iter()
                            .map(|o| o.project.as_str())
                            .collect();
                        if !global_projects.contains(origin.project.as_str()) {
                            diverging_projects.insert(origin.project.clone());
                        }
                    }
                }
            }

            if diverging_projects.len() >= self.project_threshold {
                let projects: Vec<String> = diverging_projects.into_iter().collect();
                flags.push(DivergenceFlag {
                    entry: global_entry.clone(),
                    diverging_count: projects.len(),
                    diverging_projects: projects,
                });
            }
        }

        flags
    }
}
```

- [ ] **Step 8: Run the tests**

Run: `cargo test --test contradiction_test --test supersede_test --test divergence_test`
Expected: All 5 tests pass

- [ ] **Step 9: Commit**

```bash
git add src/evolution/ tests/contradiction_test.rs tests/supersede_test.rs tests/divergence_test.rs
git commit -m "feat: add knowledge evolution — contradiction, supersession, divergence detection"
```

---

## Task 9: `mnemosyne init` Command

**Files:**
- Modify: `src/commands/init.rs`
- Modify: `src/main.rs`
- Create: `tests/init_test.rs`
- Create: `defaults/gitignore`

- [ ] **Step 1: Write the failing tests**

```rust
// tests/init_test.rs
use std::fs;
use tempfile::TempDir;
use mnemosyne::commands::init::run_init;

#[test]
fn test_init_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();
    let mnemosyne_dir = tmp.path().join(".mnemosyne");

    run_init(&mnemosyne_dir, None).unwrap();

    assert!(mnemosyne_dir.exists());
    assert!(mnemosyne_dir.join("knowledge").exists());
    assert!(mnemosyne_dir.join("knowledge/languages").exists());
    assert!(mnemosyne_dir.join("knowledge/domains").exists());
    assert!(mnemosyne_dir.join("knowledge/tools").exists());
    assert!(mnemosyne_dir.join("knowledge/techniques").exists());
    assert!(mnemosyne_dir.join("knowledge/projects").exists());
    assert!(mnemosyne_dir.join("archive").exists());
    assert!(mnemosyne_dir.join("cache").exists());
    assert!(mnemosyne_dir.join("config.yml").exists());
    assert!(mnemosyne_dir.join(".gitignore").exists());
}

#[test]
fn test_init_creates_valid_config() {
    let tmp = TempDir::new().unwrap();
    let mnemosyne_dir = tmp.path().join(".mnemosyne");

    run_init(&mnemosyne_dir, None).unwrap();

    let config = mnemosyne::config::Config::load(&mnemosyne_dir).unwrap();
    assert!(config.language_profiles.contains_key("rust"));
}

#[test]
fn test_init_refuses_if_already_exists() {
    let tmp = TempDir::new().unwrap();
    let mnemosyne_dir = tmp.path().join(".mnemosyne");

    run_init(&mnemosyne_dir, None).unwrap();
    let result = run_init(&mnemosyne_dir, None);

    assert!(result.is_err());
}

#[test]
fn test_init_creates_gitignore() {
    let tmp = TempDir::new().unwrap();
    let mnemosyne_dir = tmp.path().join(".mnemosyne");

    run_init(&mnemosyne_dir, None).unwrap();

    let gitignore = fs::read_to_string(mnemosyne_dir.join(".gitignore")).unwrap();
    assert!(gitignore.contains("cache/"));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test init_test`
Expected: Compilation error — `run_init` not found

- [ ] **Step 3: Create defaults/gitignore**

```
# Derived data — rebuilt automatically
cache/
```

- [ ] **Step 4: Implement run_init**

```rust
// src/commands/init.rs
use crate::config::Config;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Initialize a new Mnemosyne knowledge store.
pub fn run_init(mnemosyne_dir: &Path, from_repo: Option<&str>) -> Result<()> {
    if mnemosyne_dir.exists() {
        bail!(
            "Directory already exists: {}. Use a different path or remove it first.",
            mnemosyne_dir.display()
        );
    }

    if let Some(repo_url) = from_repo {
        clone_from_repo(mnemosyne_dir, repo_url)?;
    } else {
        create_fresh(mnemosyne_dir)?;
    }

    Ok(())
}

fn create_fresh(dir: &Path) -> Result<()> {
    // Create directory structure
    let dirs = [
        "knowledge/languages",
        "knowledge/domains",
        "knowledge/tools",
        "knowledge/techniques",
        "knowledge/projects",
        "archive",
        "cache",
    ];

    for subdir in &dirs {
        fs::create_dir_all(dir.join(subdir))
            .with_context(|| format!("Failed to create {}", subdir))?;
    }

    // Write default config
    let config = Config::default();
    config.save(dir)?;

    // Write .gitignore
    let gitignore = "# Derived data — rebuilt automatically\ncache/\n";
    fs::write(dir.join(".gitignore"), gitignore)?;

    // Initialize git repo
    let git_result = Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output();

    if let Ok(output) = git_result {
        if output.status.success() {
            // Make initial commit
            let _ = Command::new("git")
                .args(["add", "."])
                .current_dir(dir)
                .output();
            let _ = Command::new("git")
                .args(["commit", "-m", "Initialize Mnemosyne knowledge store"])
                .current_dir(dir)
                .output();
        }
    }

    Ok(())
}

fn clone_from_repo(dir: &Path, repo_url: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["clone", repo_url, &dir.to_string_lossy()])
        .output()
        .context("Failed to run git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git clone failed: {}", stderr);
    }

    Ok(())
}
```

- [ ] **Step 5: Wire init into main.rs**

Update the `Commands::Init` match arm in `src/main.rs`:

```rust
Commands::Init { from } => {
    let mnemosyne_dir = dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".mnemosyne");
    commands::init::run_init(&mnemosyne_dir, from.as_deref())?;
    println!("Mnemosyne initialized at {}", mnemosyne_dir.display());
}
```

Add `dirs` to Cargo.toml dependencies:

```toml
dirs = "6"
```

Add to the top of main.rs:

```rust
mod commands { pub use mnemosyne::commands::*; }
```

Actually, simpler — import the library:

```rust
use mnemosyne::commands;
```

- [ ] **Step 6: Run the tests**

Run: `cargo test --test init_test`
Expected: All 4 tests pass

- [ ] **Step 7: Run the full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 8: Commit**

```bash
git add src/commands/init.rs src/main.rs tests/init_test.rs defaults/gitignore Cargo.toml Cargo.lock
git commit -m "feat: add mnemosyne init command"
```

---

## Task 10: `mnemosyne query` Command

**Files:**
- Modify: `src/commands/query.rs`
- Modify: `src/main.rs`
- Create: `tests/query_test.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// tests/query_test.rs
use mnemosyne::commands::query::{run_query, OutputFormat, QueryOptions};
use mnemosyne::knowledge::entry::{Confidence, Entry, Origin};
use mnemosyne::knowledge::store::KnowledgeStore;
use chrono::NaiveDate;
use std::fs;
use tempfile::TempDir;

fn setup_test_store(tmp: &TempDir) -> KnowledgeStore {
    let knowledge = tmp.path().join("knowledge");
    let archive = tmp.path().join("archive");
    fs::create_dir_all(knowledge.join("languages")).unwrap();
    fs::create_dir_all(knowledge.join("tools")).unwrap();
    fs::create_dir_all(&archive).unwrap();

    let rust_entry = r#"---
title: Rust Async Patterns
tags: [rust, async, tokio]
created: 2026-03-31
last_validated: 2026-03-31
confidence: high
origins:
  - project: test
    date: 2026-03-31
    context: "test"
supersedes: []
---

## Bounded channels

Always use bounded channels.
"#;
    fs::write(knowledge.join("languages/rust.md"), rust_entry).unwrap();

    let cargo_entry = r#"---
title: Cargo Tips
tags: [cargo, rust, tooling]
created: 2026-03-31
last_validated: 2026-03-31
confidence: medium
origins:
  - project: test
    date: 2026-03-31
    context: "test"
supersedes: []
---

## Workspace tricks

Use workspace dependencies.
"#;
    fs::write(knowledge.join("tools/cargo.md"), cargo_entry).unwrap();

    KnowledgeStore::new(knowledge, archive)
}

#[test]
fn test_query_by_text() {
    let tmp = TempDir::new().unwrap();
    let store = setup_test_store(&tmp);
    let entries = store.load_all().unwrap();

    let opts = QueryOptions {
        terms: vec!["async".to_string()],
        tags: vec![],
        format: OutputFormat::Plain,
        max_results: 10,
    };

    let output = run_query(&entries, &opts).unwrap();
    assert!(output.contains("Rust Async"));
}

#[test]
fn test_query_by_tag() {
    let tmp = TempDir::new().unwrap();
    let store = setup_test_store(&tmp);
    let entries = store.load_all().unwrap();

    let opts = QueryOptions {
        terms: vec![],
        tags: vec!["cargo".to_string()],
        format: OutputFormat::Plain,
        max_results: 10,
    };

    let output = run_query(&entries, &opts).unwrap();
    assert!(output.contains("Cargo Tips"));
}

#[test]
fn test_query_max_results() {
    let tmp = TempDir::new().unwrap();
    let store = setup_test_store(&tmp);
    let entries = store.load_all().unwrap();

    let opts = QueryOptions {
        terms: vec!["rust".to_string()],
        tags: vec![],
        format: OutputFormat::Plain,
        max_results: 1,
    };

    let output = run_query(&entries, &opts).unwrap();
    // Should only have one result
    let title_count = output.matches("title:").count();
    assert_eq!(title_count, 1);
}

#[test]
fn test_query_markdown_format() {
    let tmp = TempDir::new().unwrap();
    let store = setup_test_store(&tmp);
    let entries = store.load_all().unwrap();

    let opts = QueryOptions {
        terms: vec!["rust".to_string()],
        tags: vec![],
        format: OutputFormat::Markdown,
        max_results: 10,
    };

    let output = run_query(&entries, &opts).unwrap();
    assert!(output.contains("# ") || output.contains("## "));
}

#[test]
fn test_query_json_format() {
    let tmp = TempDir::new().unwrap();
    let store = setup_test_store(&tmp);
    let entries = store.load_all().unwrap();

    let opts = QueryOptions {
        terms: vec!["rust".to_string()],
        tags: vec![],
        format: OutputFormat::Json,
        max_results: 10,
    };

    let output = run_query(&entries, &opts).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_array());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test query_test`
Expected: Compilation error

- [ ] **Step 3: Implement run_query**

```rust
// src/commands/query.rs
use crate::knowledge::entry::Entry;
use crate::knowledge::index::{FileIndex, KnowledgeIndex, Query};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Markdown,
    Json,
    Plain,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            "plain" => Self::Plain,
            _ => Self::Markdown,
        }
    }
}

pub struct QueryOptions {
    pub terms: Vec<String>,
    pub tags: Vec<String>,
    pub format: OutputFormat,
    pub max_results: usize,
}

#[derive(Serialize)]
struct JsonEntry {
    title: String,
    tags: Vec<String>,
    confidence: String,
    body: String,
}

/// Run a query against a set of loaded entries and return formatted output.
pub fn run_query(entries: &[Entry], opts: &QueryOptions) -> Result<String> {
    let index = FileIndex::from_entries(entries.to_vec());

    let text = if opts.terms.is_empty() {
        None
    } else {
        Some(opts.terms.join(" "))
    };

    let query = Query {
        text,
        tags: opts.tags.clone(),
    };

    let results = index.search(&query);
    let limited: Vec<_> = results.into_iter().take(opts.max_results).collect();

    match opts.format {
        OutputFormat::Markdown => format_markdown(&limited),
        OutputFormat::Json => format_json(&limited),
        OutputFormat::Plain => format_plain(&limited),
    }
}

fn format_markdown(results: &[crate::knowledge::index::SearchResult]) -> Result<String> {
    if results.is_empty() {
        return Ok("No matching knowledge found.\n".to_string());
    }

    let mut out = String::new();
    for result in results {
        let entry = &result.entry;
        out.push_str(&format!("## {} ({})\n", entry.title, format_confidence(&entry.confidence)));
        out.push_str(&format!("Tags: {}\n\n", entry.tags.join(", ")));
        out.push_str(&entry.body);
        out.push_str("\n\n---\n\n");
    }
    Ok(out)
}

fn format_json(results: &[crate::knowledge::index::SearchResult]) -> Result<String> {
    let json_entries: Vec<JsonEntry> = results
        .iter()
        .map(|r| JsonEntry {
            title: r.entry.title.clone(),
            tags: r.entry.tags.clone(),
            confidence: format_confidence(&r.entry.confidence),
            body: r.entry.body.clone(),
        })
        .collect();

    Ok(serde_json::to_string_pretty(&json_entries)?)
}

fn format_plain(results: &[crate::knowledge::index::SearchResult]) -> Result<String> {
    if results.is_empty() {
        return Ok("No matching knowledge found.\n".to_string());
    }

    let mut out = String::new();
    for result in results {
        let entry = &result.entry;
        out.push_str(&format!(
            "title: {} | confidence: {} | tags: {}\n",
            entry.title,
            format_confidence(&entry.confidence),
            entry.tags.join(", ")
        ));
    }
    Ok(out)
}

fn format_confidence(c: &crate::knowledge::entry::Confidence) -> String {
    match c {
        crate::knowledge::entry::Confidence::High => "high".into(),
        crate::knowledge::entry::Confidence::Medium => "medium".into(),
        crate::knowledge::entry::Confidence::Low => "low".into(),
        crate::knowledge::entry::Confidence::Prospective => "prospective".into(),
    }
}
```

- [ ] **Step 4: Wire query into main.rs**

Update the `Commands::Query` match arm:

```rust
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
    let max_results = max_tokens.map(|t| t / 500).unwrap_or(10); // rough estimate

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
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --test query_test`
Expected: All 5 tests pass

- [ ] **Step 6: Commit**

```bash
git add src/commands/query.rs src/main.rs tests/query_test.rs
git commit -m "feat: add mnemosyne query command with text, tag, and context search"
```

---

## Task 11: `mnemosyne status` Command

**Files:**
- Modify: `src/commands/status.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Implement run_status**

```rust
// src/commands/status.rs
use crate::knowledge::entry::{Confidence, Entry};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

pub fn run_status(entries: &[Entry], mnemosyne_dir: &Path) -> Result<String> {
    let mut out = String::new();

    out.push_str(&format!("{}\n\n", "Mnemosyne Knowledge Base".bold()));
    out.push_str(&format!("Location: {}\n", mnemosyne_dir.display()));
    out.push_str(&format!("Total entries: {}\n\n", entries.len()));

    // Entries by axis (top-level directory under knowledge/)
    let knowledge_root = mnemosyne_dir.join("knowledge");
    let mut by_axis: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        if let Some(ref path) = entry.file_path {
            if let Ok(relative) = path.strip_prefix(&knowledge_root) {
                if let Some(first) = relative.components().next() {
                    let axis = first.as_os_str().to_string_lossy().to_string();
                    *by_axis.entry(axis).or_insert(0) += 1;
                }
            }
        }
    }

    out.push_str(&format!("{}\n", "Entries by axis:".bold()));
    let mut axes: Vec<_> = by_axis.iter().collect();
    axes.sort_by_key(|(name, _)| name.clone());
    for (axis, count) in axes {
        out.push_str(&format!("  {}: {}\n", axis, count));
    }

    // Entries by confidence
    let mut by_confidence: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        let key = match entry.confidence {
            Confidence::High => "high",
            Confidence::Medium => "medium",
            Confidence::Low => "low",
            Confidence::Prospective => "prospective",
        };
        *by_confidence.entry(key.to_string()).or_insert(0) += 1;
    }

    out.push_str(&format!("\n{}\n", "Entries by confidence:".bold()));
    for level in &["high", "medium", "low", "prospective"] {
        let count = by_confidence.get(*level).unwrap_or(&0);
        if *count > 0 {
            out.push_str(&format!("  {}: {}\n", level, count));
        }
    }

    // Unique origin projects
    let mut projects: Vec<String> = entries
        .iter()
        .flat_map(|e| e.origins.iter().map(|o| o.project.clone()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    projects.sort();

    out.push_str(&format!("\n{}\n", "Origin projects:".bold()));
    for project in &projects {
        out.push_str(&format!("  {}\n", project));
    }

    Ok(out)
}
```

- [ ] **Step 2: Wire status into main.rs**

```rust
Commands::Status => {
    let mnemosyne_dir = dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".mnemosyne");
    let store = knowledge::store::KnowledgeStore::new(
        mnemosyne_dir.join("knowledge"),
        mnemosyne_dir.join("archive"),
    );
    let entries = store.load_all()?;
    print!("{}", commands::status::run_status(&entries, &mnemosyne_dir)?);
}
```

- [ ] **Step 3: Verify it compiles and runs**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add src/commands/status.rs src/main.rs
git commit -m "feat: add mnemosyne status command"
```

---

## Task 12: `mnemosyne promote` Command

**Files:**
- Modify: `src/commands/promote.rs`
- Modify: `src/main.rs`
- Create: `tests/promote_test.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// tests/promote_test.rs
use chrono::NaiveDate;
use mnemosyne::commands::promote::{build_new_entry, Resolution};
use mnemosyne::knowledge::entry::{Confidence, Entry, Origin};
use mnemosyne::knowledge::store::KnowledgeStore;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_build_new_entry() {
    let entry = build_new_entry(
        "Bounded channels prevent backpressure",
        &["rust", "async", "channels"],
        Confidence::High,
        "apianyware-macos",
        "Racket FFI bridge work",
        "Always use bounded channels in tokio.",
    );

    assert_eq!(entry.title, "Bounded channels prevent backpressure");
    assert_eq!(entry.tags, vec!["rust", "async", "channels"]);
    assert_eq!(entry.confidence, Confidence::High);
    assert_eq!(entry.origins[0].project, "apianyware-macos");
    assert!(entry.body.contains("Always use bounded channels"));
}

#[test]
fn test_promote_creates_file() {
    let tmp = TempDir::new().unwrap();
    let knowledge = tmp.path().join("knowledge");
    let archive = tmp.path().join("archive");
    fs::create_dir_all(knowledge.join("techniques")).unwrap();
    fs::create_dir_all(&archive).unwrap();

    let store = KnowledgeStore::new(knowledge.clone(), archive);

    let mut entry = build_new_entry(
        "TDD Patterns",
        &["tdd", "testing"],
        Confidence::High,
        "test-project",
        "test context",
        "Write tests first.",
    );

    store.create_entry("techniques", "tdd-patterns.md", &mut entry).unwrap();

    assert!(knowledge.join("techniques/tdd-patterns.md").exists());

    // Verify it can be loaded back
    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].title, "TDD Patterns");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test promote_test`
Expected: Compilation error

- [ ] **Step 3: Implement promote module**

```rust
// src/commands/promote.rs
use crate::evolution::contradiction::ContradictionDetector;
use crate::knowledge::entry::{Confidence, Entry, Origin, Tag};
use anyhow::Result;
use chrono::Local;

/// The developer's resolution when a contradiction is detected.
pub enum Resolution {
    /// Replace the existing entry with the new understanding.
    Supersede { reason: String },
    /// Both are valid in different contexts.
    Coexist { scope_note: String },
    /// The new observation was wrong — don't promote.
    Discard,
    /// Edit both entries to capture the nuance.
    Refine,
}

/// Build a new knowledge entry ready for promotion.
pub fn build_new_entry(
    title: &str,
    tags: &[&str],
    confidence: Confidence,
    project: &str,
    context: &str,
    body: &str,
) -> Entry {
    let today = Local::now().date_naive();
    Entry {
        title: title.to_string(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        created: today,
        last_validated: today,
        confidence,
        source: None,
        origins: vec![Origin {
            project: project.to_string(),
            date: today,
            context: context.to_string(),
        }],
        supersedes: vec![],
        body: format!(
            "## {}\n\n**{}:** {}\n",
            title,
            today,
            body.trim()
        ),
        file_path: None,
    }
}

/// Check for contradictions against existing entries.
pub fn check_contradictions(existing: &[Entry], new_entry: &Entry) -> Vec<crate::evolution::contradiction::PotentialContradiction> {
    let detector = ContradictionDetector::new(0.5);
    detector.detect(existing, new_entry)
}

/// Suggest which axis directory a new entry should live in based on its tags.
pub fn suggest_axis(tags: &[Tag]) -> &'static str {
    // Simple heuristic based on common tag patterns
    let tag_set: std::collections::HashSet<&str> = tags.iter().map(|t| t.as_str()).collect();

    // Check for language-specific tags
    let languages = [
        "rust", "python", "haskell", "ocaml", "prolog", "mercury",
        "scheme", "racket", "common-lisp", "smalltalk", "idris", "swift",
        "javascript", "typescript", "go", "java", "c", "cpp",
    ];
    if tag_set.iter().any(|t| languages.contains(t)) {
        return "languages";
    }

    // Check for tool-specific tags
    let tools = [
        "cargo", "git", "docker", "npm", "pip", "stack", "dune",
        "xcode", "vscode", "neovim",
    ];
    if tag_set.iter().any(|t| tools.contains(t)) {
        return "tools";
    }

    // Check for domain tags
    let domains = [
        "macos", "appkit", "web", "database", "networking", "cloud",
        "mobile", "embedded", "api",
    ];
    if tag_set.iter().any(|t| domains.contains(t)) {
        return "domains";
    }

    // Default to techniques
    "techniques"
}

/// Generate a filename from a title.
pub fn title_to_filename(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse multiple hyphens and trim
    let mut result = String::new();
    let mut prev_hyphen = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    let trimmed = result.trim_matches('-').to_string();
    format!("{}.md", trimmed)
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test promote_test`
Expected: All 2 tests pass

- [ ] **Step 5: Wire promote into main.rs**

The promote command is interactive (reads from stdin). Wire up the basic structure:

```rust
Commands::Promote { tags, origin } => {
    let mnemosyne_dir = dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".mnemosyne");
    let store = knowledge::store::KnowledgeStore::new(
        mnemosyne_dir.join("knowledge"),
        mnemosyne_dir.join("archive"),
    );
    let entries = store.load_all()?;

    // Interactive promotion flow
    println!("{}", "Mnemosyne — Promote to Global Knowledge".bold());
    println!();

    // Get title
    println!("Title for this knowledge entry:");
    let mut title = String::new();
    std::io::stdin().read_line(&mut title)?;
    let title = title.trim();

    // Get tags
    let tags: Vec<String> = if let Some(ref t) = tags {
        t.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        println!("Tags (comma-separated):");
        let mut tag_input = String::new();
        std::io::stdin().read_line(&mut tag_input)?;
        tag_input.split(',').map(|s| s.trim().to_string()).collect()
    };

    // Get origin
    let origin = origin.unwrap_or_else(|| {
        println!("Origin project:");
        let mut o = String::new();
        std::io::stdin().read_line(&mut o).unwrap();
        o.trim().to_string()
    });

    // Get body
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
        title, &tag_refs, knowledge::entry::Confidence::High, &origin, "manual promotion", &body,
    );

    // Check for contradictions
    let contradictions = commands::promote::check_contradictions(&entries, &new_entry);
    if !contradictions.is_empty() {
        println!("\n{}", "⚠ Potential contradictions detected:".yellow().bold());
        for c in &contradictions {
            println!("  {} (overlap: {:.0}%)", c.existing.title, c.overlap_score * 100.0);
        }
        println!("\n[s]upersede  [c]oexist  [d]iscard  [r]efine");
        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice)?;
        match choice.trim().chars().next() {
            Some('d') => {
                println!("Discarded.");
                return Ok(());
            }
            // Other resolutions would be handled here
            _ => {}
        }
    }

    // Save the entry
    let axis = commands::promote::suggest_axis(&new_entry.tags);
    let filename = commands::promote::title_to_filename(title);
    let mut entry = new_entry;
    store.create_entry(axis, &filename, &mut entry)?;
    println!("\n✓ Promoted to knowledge/{}/{}", axis, filename);
}
```

- [ ] **Step 6: Run the full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add src/commands/promote.rs src/main.rs tests/promote_test.rs
git commit -m "feat: add mnemosyne promote command with contradiction detection"
```

---

## Task 13: `mnemosyne curate` and `mnemosyne explore` Commands

**Files:**
- Modify: `src/commands/curate.rs`
- Modify: `src/commands/explore.rs`
- Modify: `src/main.rs`

These are interactive commands that guide the user through reflection and exploration sessions. The core logic (divergence detection, gap analysis) is already implemented in the evolution and index modules. These commands wire it up with interactive prompts.

- [ ] **Step 1: Implement curate command**

```rust
// src/commands/curate.rs
use crate::evolution::divergence::{DivergenceDetector, DivergenceFlag};
use crate::knowledge::entry::{Confidence, Entry};
use crate::knowledge::store::KnowledgeStore;
use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use std::io::{self, Write};

/// Run an interactive curation session.
pub fn run_curate(store: &KnowledgeStore, entries: &[Entry]) -> Result<()> {
    println!("{}\n", "Mnemosyne — Reflective Curation Session".bold());

    // 1. Check for divergence
    let recent: Vec<Entry> = entries
        .iter()
        .filter(|e| {
            let cutoff = Local::now().date_naive() - chrono::Duration::days(90);
            e.origins.iter().any(|o| o.date > cutoff)
        })
        .cloned()
        .collect();

    let detector = DivergenceDetector::new(0.5, 2);
    let divergences = detector.detect(entries, &recent);

    if !divergences.is_empty() {
        println!("{}\n", "Entries with potential divergence:".yellow().bold());
        for flag in &divergences {
            println!(
                "  {} — {} diverging projects: {}",
                flag.entry.title.bold(),
                flag.diverging_count,
                flag.diverging_projects.join(", ")
            );
        }
        println!();
    }

    // 2. Present entries for review grouped by recent activity areas
    let mut active_tags: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for entry in &recent {
        for tag in &entry.tags {
            *active_tags.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let mut tag_counts: Vec<_> = active_tags.into_iter().collect();
    tag_counts.sort_by(|a, b| b.1.cmp(&a.1));

    if !tag_counts.is_empty() {
        let top_areas: Vec<String> = tag_counts.iter().take(5).map(|(t, c)| format!("{} ({})", t, c)).collect();
        println!("Areas of recent activity: {}\n", top_areas.join(", "));
    }

    // 3. Interactive review loop
    let review_entries: Vec<&Entry> = if !divergences.is_empty() {
        divergences.iter().map(|d| &d.entry).collect()
    } else {
        // Review entries related to top active areas
        let top_tags: Vec<String> = tag_counts.iter().take(3).map(|(t, _)| t.clone()).collect();
        entries
            .iter()
            .filter(|e| e.tags.iter().any(|t| top_tags.contains(t)))
            .take(10)
            .collect()
    };

    if review_entries.is_empty() {
        println!("No entries to review at this time.");
        return Ok(());
    }

    println!("{} entries to review:\n", review_entries.len());

    for (i, entry) in review_entries.iter().enumerate() {
        println!(
            "{}. {} [{}] tags: {}",
            i + 1,
            entry.title.bold(),
            format_confidence(&entry.confidence),
            entry.tags.join(", ")
        );
        println!("   Last validated: {}", entry.last_validated);
        println!();
        println!("   [v]alidate  [s]upersede  [r]efine  [p]rune  [n]ext");
        print!("   > ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim().chars().next() {
            Some('v') => {
                // Update last_validated
                let mut updated = (*entry).clone();
                updated.last_validated = Local::now().date_naive();
                store.save_entry(&updated)?;
                println!("   ✓ Validated\n");
            }
            Some('p') => {
                println!("   Reason for pruning:");
                let mut reason = String::new();
                io::stdin().read_line(&mut reason)?;
                store.archive_entry(entry, reason.trim())?;
                println!("   ✓ Archived\n");
            }
            Some('n') | None => {
                println!("   Skipped\n");
            }
            _ => {
                println!("   Skipped (not yet implemented)\n");
            }
        }
    }

    println!("{}", "Curation session complete.".green());
    Ok(())
}

fn format_confidence(c: &Confidence) -> String {
    match c {
        Confidence::High => "high".to_string(),
        Confidence::Medium => "medium".to_string(),
        Confidence::Low => "low".to_string(),
        Confidence::Prospective => "prospective".to_string(),
    }
}
```

- [ ] **Step 2: Implement explore command**

```rust
// src/commands/explore.rs
use crate::knowledge::entry::{Confidence, Entry};
use crate::knowledge::store::KnowledgeStore;
use crate::knowledge::tags::TagMatcher;
use anyhow::Result;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

/// Run an interactive knowledge exploration session.
pub fn run_explore(store: &KnowledgeStore, entries: &[Entry]) -> Result<()> {
    println!("{}\n", "Mnemosyne — Knowledge Exploration Session".bold());

    // 1. Gap analysis
    println!("{}\n", "Gap Analysis".bold().underline());
    let gaps = find_gaps(entries);
    if !gaps.is_empty() {
        for gap in &gaps {
            println!("  • {}", gap);
        }
        println!();
    } else {
        println!("  No obvious gaps detected.\n");
    }

    // 2. Open questions — entries with low/prospective confidence
    let open: Vec<&Entry> = entries
        .iter()
        .filter(|e| matches!(e.confidence, Confidence::Low | Confidence::Prospective))
        .collect();

    if !open.is_empty() {
        println!("{}\n", "Open Questions / Prospective Knowledge".bold().underline());
        for entry in &open {
            let label = match entry.confidence {
                Confidence::Prospective => "prospective",
                Confidence::Low => "low confidence",
                _ => "",
            };
            println!("  • {} [{}]", entry.title, label);
        }
        println!();
    }

    // 3. Tag clusters that might benefit from synthesis
    let clusters = find_tag_clusters(entries);
    if !clusters.is_empty() {
        println!("{}\n", "Tag Clusters (may benefit from synthesis)".bold().underline());
        for (tags, count) in &clusters {
            println!("  • {} — {} entries", tags, count);
        }
        println!();
    }

    // 4. Interactive: ask if user wants to add knowledge for any gap
    println!("Would you like to explore any of these areas? (Enter a topic, or 'q' to quit)");
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input == "q" || input.is_empty() {
        println!("{}", "\nExploration session complete.".green());
        return Ok(());
    }

    println!("\nTell me about your experience with '{}':", input);
    println!("(Type your thoughts, end with an empty line)\n");

    let mut body = String::new();
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        body.push_str(&line);
    }

    if !body.trim().is_empty() {
        println!("\nSuggested tags for this knowledge:");
        let suggested_tags: Vec<String> = input
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        println!("  {}", suggested_tags.join(", "));
        println!("\nSave as [h]igh, [m]edium, [l]ow, or [p]rospective confidence? (or [d]iscard)");
        print!("> ");
        io::stdout().flush()?;

        let mut conf_input = String::new();
        io::stdin().read_line(&mut conf_input)?;

        let confidence = match conf_input.trim().chars().next() {
            Some('h') => Confidence::High,
            Some('m') => Confidence::Medium,
            Some('l') => Confidence::Low,
            Some('p') => Confidence::Prospective,
            Some('d') => {
                println!("Discarded.");
                return Ok(());
            }
            _ => Confidence::Medium,
        };

        let mut entry = crate::commands::promote::build_new_entry(
            input,
            &suggested_tags.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            confidence,
            "global",
            "exploration session",
            &body,
        );

        let axis = crate::commands::promote::suggest_axis(&entry.tags);
        let filename = crate::commands::promote::title_to_filename(input);
        store.create_entry(axis, &filename, &mut entry)?;
        println!("\n✓ Saved to knowledge/{}/{}", axis, filename);
    }

    println!("{}", "\nExploration session complete.".green());
    Ok(())
}

fn find_gaps(entries: &[Entry]) -> Vec<String> {
    let mut gaps = Vec::new();

    // Collect all tags to see what areas exist
    let all_tags: HashSet<&str> = entries.iter().flat_map(|e| e.tags.iter().map(|t| t.as_str())).collect();

    // Collect unique origin projects
    let projects: HashSet<&str> = entries
        .iter()
        .flat_map(|e| e.origins.iter().map(|o| o.project.as_str()))
        .collect();

    // Check for languages used but with few entries
    let language_tags = ["rust", "python", "haskell", "ocaml", "swift", "racket", "scheme", "prolog"];
    let mut language_counts: HashMap<&str, usize> = HashMap::new();
    for entry in entries {
        for tag in &entry.tags {
            if language_tags.contains(&tag.as_str()) {
                *language_counts.entry(tag.as_str()).or_insert(0) += 1;
            }
        }
    }

    for (lang, count) in &language_counts {
        if *count < 3 {
            gaps.push(format!(
                "You have knowledge tagged '{}' but only {} entries — could be expanded",
                lang, count
            ));
        }
    }

    // Check for tags that appear only once
    let mut tag_counts: HashMap<&str, usize> = HashMap::new();
    for entry in entries {
        for tag in &entry.tags {
            *tag_counts.entry(tag.as_str()).or_insert(0) += 1;
        }
    }

    let singletons: Vec<&&str> = tag_counts.iter().filter(|(_, c)| **c == 1).map(|(t, _)| t).collect();
    if singletons.len() > 3 {
        gaps.push(format!(
            "{} tags appear in only 1 entry — consider expanding or consolidating",
            singletons.len()
        ));
    }

    if projects.len() > 3 && entries.len() < projects.len() * 2 {
        gaps.push("Knowledge entries are sparse relative to the number of projects — consider promoting more learnings".to_string());
    }

    gaps
}

fn find_tag_clusters(entries: &[Entry]) -> Vec<(String, usize)> {
    // Find pairs of tags that frequently co-occur
    let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();

    for entry in entries {
        let mut sorted_tags = entry.tags.clone();
        sorted_tags.sort();
        for i in 0..sorted_tags.len() {
            for j in (i + 1)..sorted_tags.len() {
                let pair = (sorted_tags[i].clone(), sorted_tags[j].clone());
                *pair_counts.entry(pair).or_insert(0) += 1;
            }
        }
    }

    let mut clusters: Vec<(String, usize)> = pair_counts
        .into_iter()
        .filter(|(_, count)| *count >= 3)
        .map(|((a, b), count)| (format!("{} + {}", a, b), count))
        .collect();

    clusters.sort_by(|a, b| b.1.cmp(&a.1));
    clusters.truncate(5);
    clusters
}
```

- [ ] **Step 3: Wire curate and explore into main.rs**

```rust
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
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 5: Run the full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src/commands/curate.rs src/commands/explore.rs src/main.rs
git commit -m "feat: add mnemosyne curate and explore commands"
```

---

## Task 14: `mnemosyne install claude-code` Command

**Files:**
- Modify: `src/commands/install.rs`
- Create: `tests/install_test.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// tests/install_test.rs
use mnemosyne::commands::install::run_install_claude_code;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_install_claude_code_copies_plugin() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("adapters/claude-code");
    let target = tmp.path().join("plugins/observational-memory");

    // Create source plugin structure
    fs::create_dir_all(source.join("skills")).unwrap();
    fs::create_dir_all(source.join("references")).unwrap();
    fs::write(source.join("plugin.json"), r#"{"name": "observational-memory"}"#).unwrap();
    fs::write(source.join("skills/begin-work.md"), "# begin-work skill").unwrap();
    fs::write(source.join("references/guide.md"), "# guide").unwrap();

    run_install_claude_code(&source, &target).unwrap();

    assert!(target.join("plugin.json").exists());
    assert!(target.join("skills/begin-work.md").exists());
    assert!(target.join("references/guide.md").exists());
}

#[test]
fn test_install_preserves_existing_project_skills() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("adapters/claude-code");
    let target = tmp.path().join("plugins/observational-memory");

    // Create source plugin
    fs::create_dir_all(source.join("skills")).unwrap();
    fs::write(source.join("plugin.json"), r#"{"name": "observational-memory"}"#).unwrap();
    fs::write(source.join("skills/begin-work.md"), "# updated begin-work").unwrap();

    // Create target with a user's custom skill
    fs::create_dir_all(target.join("skills")).unwrap();
    fs::write(target.join("skills/my-custom.md"), "# my custom skill").unwrap();
    fs::write(target.join("skills/begin-work.md"), "# old begin-work").unwrap();

    run_install_claude_code(&source, &target).unwrap();

    // Custom skill should be preserved
    let custom = fs::read_to_string(target.join("skills/my-custom.md")).unwrap();
    assert_eq!(custom, "# my custom skill");

    // Plugin skill should be updated
    let updated = fs::read_to_string(target.join("skills/begin-work.md")).unwrap();
    assert_eq!(updated, "# updated begin-work");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test install_test`
Expected: Compilation error

- [ ] **Step 3: Implement run_install_claude_code**

```rust
// src/commands/install.rs
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Install the Claude Code adapter plugin.
/// `source` — the adapters/claude-code directory in the Mnemosyne installation.
/// `target` — the plugin directory (typically ~/.claude/plugins/observational-memory/).
pub fn run_install_claude_code(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)
        .with_context(|| format!("Failed to create {}", target.display()))?;

    // Copy all files from source, preserving directory structure
    copy_dir_recursive(source, target)?;

    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    for entry in fs::read_dir(source).context("Failed to read source directory")? {
        let entry = entry?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let target_path = target.join(&file_name);

        if source_path.is_dir() {
            fs::create_dir_all(&target_path)?;
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test install_test`
Expected: All 2 tests pass

- [ ] **Step 5: Wire install into main.rs**

```rust
Commands::Install { adapter } => {
    match adapter.as_str() {
        "claude-code" => {
            // Determine source: the adapters directory relative to the binary
            // For installed builds, we bundle the plugin files
            let plugin_target = dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".claude/plugins/observational-memory");

            // Find the adapter source — check relative to the executable first
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
                .ok_or_else(|| anyhow::anyhow!(
                    "Could not find adapter files. Run from the Mnemosyne repo directory."
                ))?;

            commands::install::run_install_claude_code(source, &plugin_target)?;
            println!("✓ Claude Code plugin installed to {}", plugin_target.display());
        }
        other => {
            println!("Unknown adapter: {}. Available: claude-code", other);
        }
    }
}
```

- [ ] **Step 6: Run the full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add src/commands/install.rs src/main.rs tests/install_test.rs
git commit -m "feat: add mnemosyne install claude-code command"
```

---

## Task 15: Claude Code Plugin — Skills and References

**Files:**
- Create: `adapters/claude-code/plugin.json`
- Create: `adapters/claude-code/skills/begin-work.md`
- Create: `adapters/claude-code/skills/reflect.md`
- Create: `adapters/claude-code/skills/create-plan.md`
- Create: `adapters/claude-code/skills/setup-knowledge.md`
- Create: `adapters/claude-code/skills/curate-global.md`
- Create: `adapters/claude-code/skills/promote-global.md`
- Create: `adapters/claude-code/skills/explore-knowledge.md`
- Create: `adapters/claude-code/references/observational-memory-guide.md`
- Create: `adapters/claude-code/references/plan-format.md`
- Create: `adapters/claude-code/references/coding-conventions.md`
- Create: `adapters/claude-code/references/global-knowledge-guide.md`

This task creates the plugin files. The existing plugin at `~/.claude/plugins/observational-memory/` serves as the base; skills are updated to add global knowledge integration.

- [ ] **Step 1: Create plugin.json**

```json
{
  "name": "observational-memory",
  "version": "2.0.0",
  "description": "Observational memory, knowledge management, and integrated planning for LLM-driven projects — with Mnemosyne global knowledge integration",
  "skills": [
    {
      "name": "begin-work",
      "path": "skills/begin-work.md",
      "description": "Start or continue implementation work with full knowledge context"
    },
    {
      "name": "reflect",
      "path": "skills/reflect.md",
      "description": "Promote plan observations to the knowledge base during code review"
    },
    {
      "name": "setup-knowledge",
      "path": "skills/setup-knowledge.md",
      "description": "Scaffold the observational memory knowledge system for a new project"
    },
    {
      "name": "create-plan",
      "path": "skills/create-plan.md",
      "description": "Create a multi-session plan with built-in observational memory"
    },
    {
      "name": "curate-global",
      "path": "skills/curate-global.md",
      "description": "Reflective curation of global knowledge — validate, supersede, or prune entries"
    },
    {
      "name": "promote-global",
      "path": "skills/promote-global.md",
      "description": "Promote a learning to the global Mnemosyne knowledge base"
    },
    {
      "name": "explore-knowledge",
      "path": "skills/explore-knowledge.md",
      "description": "Interactive knowledge exploration — gap analysis, horizon scanning, open questions"
    }
  ]
}
```

- [ ] **Step 2: Create updated begin-work.md skill**

Copy the content from the existing `~/.claude/plugins/observational-memory/skills/begin-work.md` and add the global knowledge step after step 3. The new step (3a):

```markdown
## 3a. Load global knowledge (if Mnemosyne CLI available)

Check if the `mnemosyne` CLI is installed by running: `which mnemosyne`

If available, run: `mnemosyne query --context --format markdown`

This returns relevant global knowledge inferred from the current project's languages,
tools, and dependencies. Include it in the summary under a "### Global knowledge loaded"
section.

If the CLI is not installed, skip this step silently — do not warn or error.
```

- [ ] **Step 3: Create updated reflect.md skill**

Copy the existing reflect skill and add step 8:

```markdown
## 8. Offer global promotion

Check if the `mnemosyne` CLI is installed by running: `which mnemosyne`

If available, for each observation promoted to per-project knowledge in this session:

1. Ask: "This learning may apply beyond this project. Promote to global Mnemosyne?"
2. If the user says yes, run: `mnemosyne promote --tags <inferred-tags> --origin <project-name>`
3. The CLI handles contradiction detection and the interactive resolution flow.

If the CLI is not installed, skip this step silently.
```

- [ ] **Step 4: Create remaining skill files**

Create each file with the appropriate skill content:
- `create-plan.md` — copy from existing plugin (unchanged)
- `setup-knowledge.md` — copy from existing, add step to run `mnemosyne init` if not already initialized
- `curate-global.md` — new skill that runs `mnemosyne curate` and guides the LLM through the curation session
- `promote-global.md` — new skill that runs `mnemosyne promote` for ad-hoc promotion
- `explore-knowledge.md` — new skill that runs gap analysis, web searches, and knowledge discussion

- [ ] **Step 5: Create reference documents**

Copy the three existing reference files and create the new `global-knowledge-guide.md`:
- `observational-memory-guide.md` — from existing plugin
- `plan-format.md` — from existing plugin
- `coding-conventions.md` — from existing plugin
- `global-knowledge-guide.md` — new document explaining:
  - What global knowledge is and how it differs from per-project
  - The two-tier model
  - How to promote, curate, and explore
  - Knowledge file format and frontmatter fields
  - Evidence-based evolution philosophy

- [ ] **Step 6: Verify plugin structure is complete**

Run: `ls -R adapters/claude-code/`
Expected: All files listed in the file map are present

- [ ] **Step 7: Commit**

```bash
git add adapters/
git commit -m "feat: add Claude Code adapter plugin with global knowledge integration"
```

---

## Task 16: Documentation

**Files:**
- Modify: `README.md`
- Create: `docs/user-guide.md`
- Create: `docs/reference.md`
- Create: `docs/knowledge-format.md`
- Create: `docs/evolution-guide.md`
- Create: `docs/configuration.md`
- Create: `docs/plugin-development.md`
- Create: `docs/research-sources.md`

- [ ] **Step 1: Write README.md**

Rewrite with: project overview, philosophy (Mastra-inspired, Mnemosyne mythology), quick start (install, init, first query), feature summary, link to detailed docs.

- [ ] **Step 2: Write user-guide.md**

Complete walkthrough: installation, first-time setup, daily workflow with `/begin-work` and `/reflect`, promoting to global, curation sessions, exploration sessions, multi-machine sync via Git.

- [ ] **Step 3: Write reference.md**

CLI reference: every command with all flags, options, examples, expected output.

- [ ] **Step 4: Write knowledge-format.md**

Format spec: frontmatter fields, body format, priority codes, supersession format, prospective entries, axis conventions.

- [ ] **Step 5: Write evolution-guide.md**

Philosophy and mechanics of knowledge evolution: evidence-based (no time-based expiry), contradiction detection, supersession, divergence, reflective curation, the Socratic exploration model.

- [ ] **Step 6: Write configuration.md**

Config file reference: language profiles, context mappings, adding custom languages, dependency parser format.

- [ ] **Step 7: Write plugin-development.md**

Guide for building adapters for other harnesses: what the CLI provides, how to shell out, how to integrate context loading, graceful degradation pattern.

- [ ] **Step 8: Write research-sources.md**

Annotated bibliography: Mastra's Observational Memory, human memory models, belief revision (AGM theory), expertise accumulation, spaced retrieval, Zettelkasten, cognitive load theory. Each entry notes how it influences Mnemosyne's design.

- [ ] **Step 9: Commit**

```bash
git add README.md docs/
git commit -m "docs: add complete documentation suite"
```

---

## Task 17: Integration Test and Final Cleanup

**Files:**
- Modify: `src/main.rs` (final polish)
- Run all tests

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Expected: No warnings (or fix any that appear)

- [ ] **Step 3: Run formatter**

Run: `cargo +nightly fmt` (if nightly available, otherwise `cargo fmt`)
Expected: Code formatted

- [ ] **Step 4: Manual smoke test**

```bash
# Test the full workflow
cargo run -- init   # (use a temp location or clean up after)
cargo run -- status
cargo run -- query rust
cargo run -- --help
```

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final cleanup — clippy fixes, formatting, integration test"
```
