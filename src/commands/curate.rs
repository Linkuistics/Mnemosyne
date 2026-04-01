use crate::evolution::divergence::DivergenceDetector;
use crate::knowledge::entry::{Confidence, Entry};
use crate::knowledge::store::KnowledgeStore;
use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use std::io::{self, Write};

/// Run an interactive curation session.
pub fn run_curate(store: &KnowledgeStore, entries: &[Entry]) -> Result<()> {
    println!("{}\n", "Mnemosyne — Reflective Curation Session".bold());

    // 1. Check for divergence
    let recent: Vec<Entry> = entries
        .iter()
        .filter(|e| {
            let cutoff = Local::now().date_naive() - chrono::Duration::days(90);
            e.origins.iter().any(|o| o.date > cutoff)
        })
        .cloned()
        .collect();

    let detector = DivergenceDetector::new(0.5, 2);
    let divergences = detector.detect(entries, &recent);

    if !divergences.is_empty() {
        println!("{}\n", "Entries with potential divergence:".yellow().bold());
        for flag in &divergences {
            println!(
                "  {} — {} diverging projects: {}",
                flag.entry.title.bold(),
                flag.diverging_count,
                flag.diverging_projects.join(", ")
            );
        }
        println!();
    }

    // 2. Active areas
    let mut active_tags: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for entry in &recent {
        for tag in &entry.tags {
            *active_tags.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let mut tag_counts: Vec<_> = active_tags.into_iter().collect();
    tag_counts.sort_by(|a, b| b.1.cmp(&a.1));

    if !tag_counts.is_empty() {
        let top_areas: Vec<String> = tag_counts
            .iter()
            .take(5)
            .map(|(t, c)| format!("{} ({})", t, c))
            .collect();
        println!("Areas of recent activity: {}\n", top_areas.join(", "));
    }

    // 3. Interactive review loop
    let review_entries: Vec<&Entry> = if !divergences.is_empty() {
        divergences.iter().map(|d| &d.entry).collect()
    } else {
        let top_tags: Vec<String> = tag_counts.iter().take(3).map(|(t, _)| t.clone()).collect();
        entries
            .iter()
            .filter(|e| e.tags.iter().any(|t| top_tags.contains(t)))
            .take(10)
            .collect()
    };

    if review_entries.is_empty() {
        println!("No entries to review at this time.");
        return Ok(());
    }

    println!("{} entries to review:\n", review_entries.len());

    for (_i, entry) in review_entries.iter().enumerate() {
        println!(
            "{}. {} [{}] tags: {}",
            _i + 1,
            entry.title.bold(),
            format_confidence(&entry.confidence),
            entry.tags.join(", ")
        );
        println!("   Last validated: {}", entry.last_validated);
        println!();
        println!("   [v]alidate  [s]upersede  [r]efine  [p]rune  [n]ext");
        print!("   > ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim().chars().next() {
            Some('v') => {
                let mut updated = (*entry).clone();
                updated.last_validated = Local::now().date_naive();
                store.save_entry(&updated)?;
                println!("   ✓ Validated\n");
            }
            Some('p') => {
                println!("   Reason for pruning:");
                let mut reason = String::new();
                io::stdin().read_line(&mut reason)?;
                store.archive_entry(entry, reason.trim())?;
                println!("   ✓ Archived\n");
            }
            Some('n') | None => {
                println!("   Skipped\n");
            }
            _ => {
                println!("   Skipped (not yet implemented)\n");
            }
        }
    }

    println!("{}", "Curation session complete.".green());
    Ok(())
}

fn format_confidence(c: &Confidence) -> String {
    match c {
        Confidence::High => "high".to_string(),
        Confidence::Medium => "medium".to_string(),
        Confidence::Low => "low".to_string(),
        Confidence::Prospective => "prospective".to_string(),
    }
}
