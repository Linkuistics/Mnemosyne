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
