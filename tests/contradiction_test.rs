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
