use mnemosyne::commands::query::{run_query, OutputFormat, QueryOptions};
use mnemosyne::knowledge::store::KnowledgeStore;
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
