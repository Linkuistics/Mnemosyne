use std::collections::HashSet;

use crate::knowledge::entry::Tag;

pub struct TagMatcher;

impl TagMatcher {
    /// Count how many tags overlap between two tag lists.
    pub fn overlap_count(a: &[Tag], b: &[Tag]) -> usize {
        let set_a: HashSet<&str> = a.iter().map(|t| t.as_str()).collect();
        let set_b: HashSet<&str> = b.iter().map(|t| t.as_str()).collect();
        set_a.intersection(&set_b).count()
    }

    /// Compute a normalized overlap score between 0.0 and 1.0.
    /// Uses Jaccard similarity: |intersection| / |union|.
    pub fn overlap_score(a: &[Tag], b: &[Tag]) -> f64 {
        let set_a: HashSet<&str> = a.iter().map(|t| t.as_str()).collect();
        let set_b: HashSet<&str> = b.iter().map(|t| t.as_str()).collect();
        let union_size = set_a.union(&set_b).count();
        if union_size == 0 {
            return 0.0;
        }
        let intersection_size = set_a.intersection(&set_b).count();
        intersection_size as f64 / union_size as f64
    }

    /// Check if any tags from the query appear in the entry's tags.
    pub fn matches_any(entry_tags: &[Tag], query_tags: &[Tag]) -> bool {
        let entry_set: HashSet<&str> = entry_tags.iter().map(|t| t.as_str()).collect();
        query_tags.iter().any(|t| entry_set.contains(t.as_str()))
    }

    /// Check if all query tags appear in the entry's tags.
    pub fn matches_all(entry_tags: &[Tag], query_tags: &[Tag]) -> bool {
        let entry_set: HashSet<&str> = entry_tags.iter().map(|t| t.as_str()).collect();
        query_tags.iter().all(|t| entry_set.contains(t.as_str()))
    }
}
