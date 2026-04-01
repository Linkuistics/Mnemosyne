use mnemosyne::commands::promote::build_new_entry;
use mnemosyne::knowledge::entry::Confidence;
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
