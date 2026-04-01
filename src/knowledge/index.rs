use crate::knowledge::entry::{Confidence, Entry, Tag};
use crate::knowledge::tags::TagMatcher;

/// A query against the knowledge index.
pub struct Query {
    pub text: Option<String>,
    pub tags: Vec<Tag>,
}

/// A single search result with a relevance score.
pub struct SearchResult {
    pub entry: Entry,
    pub score: f64,
}

/// A potential contradiction between a new entry and an existing one.
pub struct PotentialContradiction {
    pub existing: Entry,
    pub overlap_score: f64,
}

/// Trait for knowledge index implementations.
pub trait KnowledgeIndex {
    fn search(&self, query: &Query) -> Vec<SearchResult>;
    fn find_contradictions(&self, entry: &Entry) -> Vec<PotentialContradiction>;
    fn find_by_tags(&self, tags: &[Tag]) -> Vec<&Entry>;
}

/// In-memory index built from scanning knowledge files.
pub struct FileIndex {
    entries: Vec<Entry>,
}

impl FileIndex {
    pub fn from_entries(entries: Vec<Entry>) -> Self {
        Self { entries }
    }

    fn score_entry(entry: &Entry, query: &Query) -> f64 {
        let mut score = 0.0;

        if !query.tags.is_empty() {
            let tag_score = TagMatcher::overlap_score(&entry.tags, &query.tags);
            score += tag_score * 10.0;
        }

        if let Some(ref text) = query.text {
            let text_lower = text.to_lowercase();
            if entry.title.to_lowercase().contains(&text_lower) {
                score += 5.0;
            }
            if entry.body.to_lowercase().contains(&text_lower) {
                score += 2.0;
            }
        }

        score *= match entry.confidence {
            Confidence::High => 1.0,
            Confidence::Medium => 0.8,
            Confidence::Low => 0.6,
            Confidence::Prospective => 0.4,
        };

        score
    }
}

impl KnowledgeIndex for FileIndex {
    fn search(&self, query: &Query) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let score = Self::score_entry(entry, query);
                if score > 0.0 {
                    Some(SearchResult {
                        entry: entry.clone(),
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    fn find_contradictions(&self, entry: &Entry) -> Vec<PotentialContradiction> {
        let threshold = 0.5;

        let mut contradictions: Vec<PotentialContradiction> = self
            .entries
            .iter()
            .filter_map(|existing| {
                let overlap = TagMatcher::overlap_score(&existing.tags, &entry.tags);
                if overlap >= threshold {
                    Some(PotentialContradiction {
                        existing: existing.clone(),
                        overlap_score: overlap,
                    })
                } else {
                    None
                }
            })
            .collect();

        contradictions.sort_by(|a, b| {
            b.overlap_score
                .partial_cmp(&a.overlap_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        contradictions
    }

    fn find_by_tags(&self, tags: &[Tag]) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|entry| TagMatcher::matches_any(&entry.tags, tags))
            .collect()
    }
}
