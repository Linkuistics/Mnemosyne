use mnemosyne::knowledge::store::KnowledgeStore;
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
