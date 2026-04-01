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
    let global = vec![make_entry_with_origins(
        "Prefer integration tests",
        &["testing", "database"],
        vec![("project-a", "2026-01-01")],
    )];

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
    let global = vec![make_entry_with_origins(
        "Prefer integration tests",
        &["testing", "database"],
        vec![("project-a", "2026-01-01")],
    )];

    let recent = vec![make_entry_with_origins(
        "Unit tests approach",
        &["testing", "database"],
        vec![("project-b", "2026-03-15")],
    )];

    let detector = DivergenceDetector::new(0.5, 2);
    let flagged = detector.detect(&global, &recent);

    assert!(flagged.is_empty()); // Only 1 diverging project, threshold is 2
}
