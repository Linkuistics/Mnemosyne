use mnemosyne::knowledge::entry::{Confidence, Entry, Origin};

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
