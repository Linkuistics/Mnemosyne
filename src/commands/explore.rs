use crate::knowledge::entry::{Confidence, Entry};
use crate::knowledge::store::KnowledgeStore;
use anyhow::Result;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

/// Run an interactive knowledge exploration session.
pub fn run_explore(store: &KnowledgeStore, entries: &[Entry]) -> Result<()> {
    println!("{}\n", "Mnemosyne — Knowledge Exploration Session".bold());

    // 1. Gap analysis
    println!("{}\n", "Gap Analysis".bold().underline());
    let gaps = find_gaps(entries);
    if !gaps.is_empty() {
        for gap in &gaps {
            println!("  • {}", gap);
        }
        println!();
    } else {
        println!("  No obvious gaps detected.\n");
    }

    // 2. Open questions
    let open: Vec<&Entry> = entries
        .iter()
        .filter(|e| matches!(e.confidence, Confidence::Low | Confidence::Prospective))
        .collect();

    if !open.is_empty() {
        println!("{}\n", "Open Questions / Prospective Knowledge".bold().underline());
        for entry in &open {
            let label = match entry.confidence {
                Confidence::Prospective => "prospective",
                Confidence::Low => "low confidence",
                _ => "",
            };
            println!("  • {} [{}]", entry.title, label);
        }
        println!();
    }

    // 3. Tag clusters
    let clusters = find_tag_clusters(entries);
    if !clusters.is_empty() {
        println!("{}\n", "Tag Clusters (may benefit from synthesis)".bold().underline());
        for (tags, count) in &clusters {
            println!("  • {} — {} entries", tags, count);
        }
        println!();
    }

    // 4. Interactive exploration
    println!("Would you like to explore any of these areas? (Enter a topic, or 'q' to quit)");
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();

    if input == "q" || input.is_empty() {
        println!("{}", "\nExploration session complete.".green());
        return Ok(());
    }

    println!("\nTell me about your experience with '{}':", input);
    println!("(Type your thoughts, end with an empty line)\n");

    let mut body = String::new();
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        body.push_str(&line);
    }

    if !body.trim().is_empty() {
        println!("\nSuggested tags for this knowledge:");
        let suggested_tags: Vec<String> = input
            .split_whitespace()
            .map(|w| {
                w.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();
        println!("  {}", suggested_tags.join(", "));
        println!("\nSave as [h]igh, [m]edium, [l]ow, or [p]rospective confidence? (or [d]iscard)");
        print!("> ");
        io::stdout().flush()?;

        let mut conf_input = String::new();
        io::stdin().read_line(&mut conf_input)?;

        let confidence = match conf_input.trim().chars().next() {
            Some('h') => Confidence::High,
            Some('m') => Confidence::Medium,
            Some('l') => Confidence::Low,
            Some('p') => Confidence::Prospective,
            Some('d') => {
                println!("Discarded.");
                return Ok(());
            }
            _ => Confidence::Medium,
        };

        let tag_refs: Vec<&str> = suggested_tags.iter().map(|s| s.as_str()).collect();
        let mut entry = crate::commands::promote::build_new_entry(
            &input,
            &tag_refs,
            confidence,
            "global",
            "exploration session",
            &body,
        );

        let axis = crate::commands::promote::suggest_axis(&entry.tags);
        let filename = crate::commands::promote::title_to_filename(&input);
        store.create_entry(axis, &filename, &mut entry)?;
        println!("\n✓ Saved to knowledge/{}/{}", axis, filename);
    }

    println!("{}", "\nExploration session complete.".green());
    Ok(())
}

fn find_gaps(entries: &[Entry]) -> Vec<String> {
    let mut gaps = Vec::new();

    let language_tags = [
        "rust", "python", "haskell", "ocaml", "swift", "racket", "scheme", "prolog",
    ];
    let mut language_counts: HashMap<&str, usize> = HashMap::new();
    for entry in entries {
        for tag in &entry.tags {
            if language_tags.contains(&tag.as_str()) {
                *language_counts.entry(tag.as_str()).or_insert(0) += 1;
            }
        }
    }

    for (lang, count) in &language_counts {
        if *count < 3 {
            gaps.push(format!(
                "You have knowledge tagged '{}' but only {} entries — could be expanded",
                lang, count
            ));
        }
    }

    let mut tag_counts: HashMap<&str, usize> = HashMap::new();
    for entry in entries {
        for tag in &entry.tags {
            *tag_counts.entry(tag.as_str()).or_insert(0) += 1;
        }
    }

    let singletons: Vec<&&str> = tag_counts
        .iter()
        .filter(|(_, c)| **c == 1)
        .map(|(t, _)| t)
        .collect();
    if singletons.len() > 3 {
        gaps.push(format!(
            "{} tags appear in only 1 entry — consider expanding or consolidating",
            singletons.len()
        ));
    }

    let projects: HashSet<&str> = entries
        .iter()
        .flat_map(|e| e.origins.iter().map(|o| o.project.as_str()))
        .collect();

    if projects.len() > 3 && entries.len() < projects.len() * 2 {
        gaps.push(
            "Knowledge entries are sparse relative to the number of projects — consider promoting more learnings"
                .to_string(),
        );
    }

    gaps
}

fn find_tag_clusters(entries: &[Entry]) -> Vec<(String, usize)> {
    let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();

    for entry in entries {
        let mut sorted_tags = entry.tags.clone();
        sorted_tags.sort();
        for i in 0..sorted_tags.len() {
            for j in (i + 1)..sorted_tags.len() {
                let pair = (sorted_tags[i].clone(), sorted_tags[j].clone());
                *pair_counts.entry(pair).or_insert(0) += 1;
            }
        }
    }

    let mut clusters: Vec<(String, usize)> = pair_counts
        .into_iter()
        .filter(|(_, count)| *count >= 3)
        .map(|((a, b), count)| (format!("{} + {}", a, b), count))
        .collect();

    clusters.sort_by(|a, b| b.1.cmp(&a.1));
    clusters.truncate(5);
    clusters
}
