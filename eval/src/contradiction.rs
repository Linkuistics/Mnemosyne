use mnemosyne::evolution::contradiction::ContradictionDetector;

use crate::corpus::Corpus;

#[derive(Debug, Clone)]
pub struct ContradictionMetrics {
    pub threshold: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub pair_count: usize,
}

pub fn evaluate_contradictions(corpus: &Corpus, threshold: f64) -> ContradictionMetrics {
    let detector = ContradictionDetector::new(threshold);
    let mut true_positives = 0;
    let mut false_positives = 0;
    let mut false_negatives = 0;

    for pair in &corpus.contradictions.pairs {
        let entry_a = corpus.entry_map.get(&pair.entry_a).map(|&i| &corpus.entries[i]);
        let entry_b = corpus.entry_map.get(&pair.entry_b).map(|&i| &corpus.entries[i]);

        let (Some(entry_a), Some(entry_b)) = (entry_a, entry_b) else {
            continue;
        };

        let results = detector.detect(std::slice::from_ref(entry_a), entry_b);
        let flagged = !results.is_empty();

        match (flagged, pair.is_contradiction) {
            (true, true) => true_positives += 1,
            (true, false) => false_positives += 1,
            (false, true) => false_negatives += 1,
            (false, false) => {} // true negative
        }
    }

    let precision = if true_positives + false_positives > 0 {
        true_positives as f64 / (true_positives + false_positives) as f64
    } else {
        0.0
    };
    let recall = if true_positives + false_negatives > 0 {
        true_positives as f64 / (true_positives + false_negatives) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    ContradictionMetrics {
        threshold,
        precision,
        recall,
        f1,
        pair_count: corpus.contradictions.pairs.len(),
    }
}

pub fn sweep_thresholds(corpus: &Corpus) -> Vec<ContradictionMetrics> {
    let mut results = Vec::new();
    let mut threshold = 0.30;
    while threshold <= 0.80 + 1e-9 {
        results.push(evaluate_contradictions(corpus, threshold));
        threshold += 0.05;
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mnemosyne::knowledge::entry::{Confidence, Entry};
    use crate::corpus::{ContradictionPair, ContradictionSet, Corpus, QuerySet};
    use std::collections::HashMap;

    fn make_entry(title: &str, tags: Vec<&str>, filename: &str) -> Entry {
        Entry {
            title: title.to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            created: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            last_validated: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            confidence: Confidence::High,
            source: None,
            origins: vec![],
            supersedes: vec![],
            body: String::new(),
            file_path: Some(std::path::PathBuf::from(filename)),
        }
    }

    fn make_test_corpus() -> Corpus {
        let entry_a = make_entry("A", vec!["rust", "async", "tokio"], "a.md");
        let entry_b = make_entry("B", vec!["rust", "async", "tokio", "cancel"], "b.md");
        let entry_c = make_entry("C", vec!["python", "typing"], "c.md");
        let entries = vec![entry_a, entry_b, entry_c];
        let entry_map: HashMap<String, usize> = [
            ("a.md".into(), 0),
            ("b.md".into(), 1),
            ("c.md".into(), 2),
        ]
        .into();

        Corpus {
            entries,
            entry_map,
            queries: QuerySet { queries: vec![] },
            contradictions: ContradictionSet {
                pairs: vec![
                    ContradictionPair {
                        entry_a: "a.md".into(),
                        entry_b: "b.md".into(),
                        is_contradiction: true,
                        note: "true contradiction".into(),
                    },
                    ContradictionPair {
                        entry_a: "a.md".into(),
                        entry_b: "c.md".into(),
                        is_contradiction: false,
                        note: "no overlap".into(),
                    },
                ],
            },
            projects: vec![],
        }
    }

    #[test]
    fn test_evaluate_known_pairs() {
        let corpus = make_test_corpus();
        let metrics = evaluate_contradictions(&corpus, 0.5);
        assert_eq!(metrics.precision, 1.0);
        assert_eq!(metrics.recall, 1.0);
        assert_eq!(metrics.f1, 1.0);
    }

    #[test]
    fn test_high_threshold_misses_contradiction() {
        let corpus = make_test_corpus();
        let metrics = evaluate_contradictions(&corpus, 0.8);
        assert_eq!(metrics.recall, 0.0);
    }

    #[test]
    fn test_sweep_returns_multiple_thresholds() {
        let corpus = make_test_corpus();
        let sweep = sweep_thresholds(&corpus);
        assert!(sweep.len() >= 10);
        assert!((sweep[0].threshold - 0.30).abs() < 1e-9);
    }
}
