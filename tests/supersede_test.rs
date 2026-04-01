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
        body: "## Use unbounded channels\n\n**2026-01-15:** Prefer unbounded for logging."
            .to_string(),
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
