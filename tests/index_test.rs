use chrono::NaiveDate;
use mnemosyne::knowledge::entry::{Confidence, Entry, Origin};
use mnemosyne::knowledge::index::{FileIndex, KnowledgeIndex, Query};

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
        make_entry(
            "Use unbounded channels",
            &["rust", "async", "channels"],
            Confidence::High,
        ),
        make_entry("Other topic", &["python"], Confidence::High),
    ];
    let index = FileIndex::from_entries(entries);

    let new_entry = make_entry(
        "Use bounded channels",
        &["rust", "async", "channels"],
        Confidence::High,
    );
    let contradictions = index.find_contradictions(&new_entry);

    assert_eq!(contradictions.len(), 1);
    assert_eq!(contradictions[0].existing.title, "Use unbounded channels");
}
