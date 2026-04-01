use crate::evolution::contradiction::ContradictionDetector;
use crate::knowledge::entry::{Confidence, Entry, Origin, Tag};
use chrono::Local;

/// The developer's resolution when a contradiction is detected.
pub enum Resolution {
    Supersede { reason: String },
    Coexist { scope_note: String },
    Discard,
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
        body: format!("## {}\n\n**{}:** {}\n", title, today, body.trim()),
        file_path: None,
    }
}

/// Check for contradictions against existing entries.
pub fn check_contradictions(
    existing: &[Entry],
    new_entry: &Entry,
) -> Vec<crate::evolution::contradiction::PotentialContradiction> {
    let detector = ContradictionDetector::new(0.5);
    detector.detect(existing, new_entry)
}

/// Suggest which axis directory a new entry should live in based on its tags.
pub fn suggest_axis(tags: &[Tag]) -> &'static str {
    let tag_set: std::collections::HashSet<&str> = tags.iter().map(|t| t.as_str()).collect();

    let languages = [
        "rust",
        "python",
        "haskell",
        "ocaml",
        "prolog",
        "mercury",
        "scheme",
        "racket",
        "common-lisp",
        "smalltalk",
        "idris",
        "swift",
        "javascript",
        "typescript",
        "go",
        "java",
        "c",
        "cpp",
    ];
    if tag_set.iter().any(|t| languages.contains(t)) {
        return "languages";
    }

    let tools = [
        "cargo", "git", "docker", "npm", "pip", "stack", "dune", "xcode", "vscode", "neovim",
    ];
    if tag_set.iter().any(|t| tools.contains(t)) {
        return "tools";
    }

    let domains = [
        "macos",
        "appkit",
        "web",
        "database",
        "networking",
        "cloud",
        "mobile",
        "embedded",
        "api",
    ];
    if tag_set.iter().any(|t| domains.contains(t)) {
        return "domains";
    }

    "techniques"
}

/// Generate a filename from a title.
pub fn title_to_filename(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
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
