use crate::knowledge::entry::Entry;
use crate::knowledge::index::{FileIndex, KnowledgeIndex, Query, SearchResult};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Markdown,
    Json,
    Plain,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            "plain" => Self::Plain,
            _ => Self::Markdown,
        }
    }
}

pub struct QueryOptions {
    pub terms: Vec<String>,
    pub tags: Vec<String>,
    pub format: OutputFormat,
    pub max_results: usize,
}

#[derive(Serialize)]
struct JsonEntry {
    title: String,
    tags: Vec<String>,
    confidence: String,
    body: String,
}

/// Run a query against a set of loaded entries and return formatted output.
pub fn run_query(entries: &[Entry], opts: &QueryOptions) -> Result<String> {
    let index = FileIndex::from_entries(entries.to_vec());

    let text = if opts.terms.is_empty() {
        None
    } else {
        Some(opts.terms.join(" "))
    };

    let query = Query {
        text,
        tags: opts.tags.clone(),
    };

    let results = index.search(&query);
    let limited: Vec<_> = results.into_iter().take(opts.max_results).collect();

    match opts.format {
        OutputFormat::Markdown => format_markdown(&limited),
        OutputFormat::Json => format_json(&limited),
        OutputFormat::Plain => format_plain(&limited),
    }
}

fn format_markdown(results: &[SearchResult]) -> Result<String> {
    if results.is_empty() {
        return Ok("No matching knowledge found.\n".to_string());
    }

    let mut out = String::new();
    for result in results {
        let entry = &result.entry;
        out.push_str(&format!(
            "## {} ({})\n",
            entry.title,
            format_confidence(&entry.confidence)
        ));
        out.push_str(&format!("Tags: {}\n\n", entry.tags.join(", ")));
        out.push_str(&entry.body);
        out.push_str("\n\n---\n\n");
    }
    Ok(out)
}

fn format_json(results: &[SearchResult]) -> Result<String> {
    let json_entries: Vec<JsonEntry> = results
        .iter()
        .map(|r| JsonEntry {
            title: r.entry.title.clone(),
            tags: r.entry.tags.clone(),
            confidence: format_confidence(&r.entry.confidence),
            body: r.entry.body.clone(),
        })
        .collect();

    Ok(serde_json::to_string_pretty(&json_entries)?)
}

fn format_plain(results: &[SearchResult]) -> Result<String> {
    if results.is_empty() {
        return Ok("No matching knowledge found.\n".to_string());
    }

    let mut out = String::new();
    for result in results {
        let entry = &result.entry;
        out.push_str(&format!(
            "title: {} | confidence: {} | tags: {}\n",
            entry.title,
            format_confidence(&entry.confidence),
            entry.tags.join(", ")
        ));
    }
    Ok(out)
}

fn format_confidence(c: &crate::knowledge::entry::Confidence) -> String {
    match c {
        crate::knowledge::entry::Confidence::High => "high".into(),
        crate::knowledge::entry::Confidence::Medium => "medium".into(),
        crate::knowledge::entry::Confidence::Low => "low".into(),
        crate::knowledge::entry::Confidence::Prospective => "prospective".into(),
    }
}
