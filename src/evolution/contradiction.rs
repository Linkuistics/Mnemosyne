use crate::knowledge::entry::Entry;
use crate::knowledge::tags::TagMatcher;

pub struct PotentialContradiction {
    pub existing: Entry,
    pub overlap_score: f64,
}

pub struct ContradictionDetector {
    threshold: f64,
}

impl ContradictionDetector {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    pub fn detect(&self, existing: &[Entry], new_entry: &Entry) -> Vec<PotentialContradiction> {
        let mut results: Vec<PotentialContradiction> = existing
            .iter()
            .filter_map(|entry| {
                let score = TagMatcher::overlap_score(&entry.tags, &new_entry.tags);
                if score >= self.threshold {
                    Some(PotentialContradiction {
                        existing: entry.clone(),
                        overlap_score: score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.overlap_score
                .partial_cmp(&a.overlap_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }
}
