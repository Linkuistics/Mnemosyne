use crate::knowledge::entry::{Confidence, Entry};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

pub fn run_status(entries: &[Entry], mnemosyne_dir: &Path) -> Result<String> {
    let mut out = String::new();

    out.push_str(&format!("{}\n\n", "Mnemosyne Knowledge Base".bold()));
    out.push_str(&format!("Location: {}\n", mnemosyne_dir.display()));
    out.push_str(&format!("Total entries: {}\n\n", entries.len()));

    // Entries by axis
    let knowledge_root = mnemosyne_dir.join("knowledge");
    let mut by_axis: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        if let Some(ref path) = entry.file_path {
            if let Ok(relative) = path.strip_prefix(&knowledge_root) {
                if let Some(first) = relative.components().next() {
                    let axis = first.as_os_str().to_string_lossy().to_string();
                    *by_axis.entry(axis).or_insert(0) += 1;
                }
            }
        }
    }

    out.push_str(&format!("{}\n", "Entries by axis:".bold()));
    let mut axes: Vec<_> = by_axis.iter().collect();
    axes.sort_by_key(|(name, _)| (*name).clone());
    for (axis, count) in axes {
        out.push_str(&format!("  {}: {}\n", axis, count));
    }

    // Entries by confidence
    let mut by_confidence: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        let key = match entry.confidence {
            Confidence::High => "high",
            Confidence::Medium => "medium",
            Confidence::Low => "low",
            Confidence::Prospective => "prospective",
        };
        *by_confidence.entry(key.to_string()).or_insert(0) += 1;
    }

    out.push_str(&format!("\n{}\n", "Entries by confidence:".bold()));
    for level in &["high", "medium", "low", "prospective"] {
        let count = by_confidence.get(*level).unwrap_or(&0);
        if *count > 0 {
            out.push_str(&format!("  {}: {}\n", level, count));
        }
    }

    // Unique origin projects
    let mut projects: Vec<String> = entries
        .iter()
        .flat_map(|e| e.origins.iter().map(|o| o.project.clone()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    projects.sort();

    out.push_str(&format!("\n{}\n", "Origin projects:".bold()));
    for project in &projects {
        out.push_str(&format!("  {}\n", project));
    }

    Ok(out)
}
