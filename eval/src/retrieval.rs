use std::collections::{HashMap, HashSet};

use mnemosyne::knowledge::entry::Tag;
use mnemosyne::knowledge::index::{FileIndex, KnowledgeIndex, Query};

use crate::corpus::Corpus;

#[derive(Debug, Clone)]
pub struct PerQueryMetrics {
    pub query_id: String,
    pub reciprocal_rank: f64,
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub ndcg_at_k: f64,
}

#[derive(Debug)]
pub struct RetrievalMetrics {
    pub mrr: f64,
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub ndcg_at_k: f64,
    pub k: usize,
    pub query_count: usize,
    pub per_query: Vec<PerQueryMetrics>,
}

/// Reciprocal rank: 1/position of the first relevant result, or 0 if none.
fn reciprocal_rank(ranked: &[String], relevant: &HashSet<String>) -> f64 {
    for (i, entry) in ranked.iter().enumerate() {
        if relevant.contains(entry) {
            return 1.0 / (i as f64 + 1.0);
        }
    }
    0.0
}

/// Fraction of top-k results that are relevant.
fn precision_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    let top_k = &ranked[..ranked.len().min(k)];
    if top_k.is_empty() {
        return 0.0;
    }
    let hits = top_k.iter().filter(|e| relevant.contains(e.as_str())).count();
    hits as f64 / k as f64
}

/// Fraction of all relevant entries appearing in top-k.
fn recall_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let top_k = &ranked[..ranked.len().min(k)];
    let hits = top_k.iter().filter(|e| relevant.contains(e.as_str())).count();
    hits as f64 / relevant.len() as f64
}

/// Normalised Discounted Cumulative Gain at k.
fn ndcg_at_k(ranked_relevance: &[u8], ideal_relevance: &[u8], k: usize) -> f64 {
    let dcg_val = dcg(ranked_relevance, k);
    let idcg_val = dcg(ideal_relevance, k);
    if idcg_val == 0.0 {
        return 0.0;
    }
    dcg_val / idcg_val
}

fn dcg(relevance: &[u8], k: usize) -> f64 {
    relevance
        .iter()
        .take(k)
        .enumerate()
        .map(|(i, &rel)| {
            let gain = (2.0_f64).powi(rel as i32) - 1.0;
            let discount = (i as f64 + 2.0).log2();
            gain / discount
        })
        .sum()
}

pub fn evaluate_retrieval(corpus: &Corpus, k: usize) -> RetrievalMetrics {
    let index = FileIndex::from_entries(corpus.entries.clone());
    let mut per_query = Vec::new();

    for query_spec in &corpus.queries.queries {
        let query = Query {
            text: Some(query_spec.text.clone()),
            tags: query_spec.tags.iter().map(|t| t.clone() as Tag).collect(),
        };

        let results = index.search(&query);
        let ranked: Vec<String> = results
            .iter()
            .filter_map(|r| {
                r.entry
                    .file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
            })
            .collect();

        let relevant_set: HashSet<String> = query_spec
            .relevant
            .iter()
            .map(|r| r.entry.clone())
            .collect();

        let relevance_map: HashMap<String, u8> = query_spec
            .relevant
            .iter()
            .map(|r| (r.entry.clone(), r.relevance))
            .collect();

        let ranked_rel: Vec<u8> = ranked
            .iter()
            .map(|e| *relevance_map.get(e).unwrap_or(&0))
            .collect();

        let mut ideal_rel: Vec<u8> = query_spec.relevant.iter().map(|r| r.relevance).collect();
        ideal_rel.sort_by(|a, b| b.cmp(a));

        let rr = reciprocal_rank(&ranked, &relevant_set);
        let p_at_k = precision_at_k(&ranked, &relevant_set, k);
        let r_at_k = recall_at_k(&ranked, &relevant_set, k);
        let ndcg = ndcg_at_k(&ranked_rel, &ideal_rel, k);

        per_query.push(PerQueryMetrics {
            query_id: query_spec.id.clone(),
            reciprocal_rank: rr,
            precision_at_k: p_at_k,
            recall_at_k: r_at_k,
            ndcg_at_k: ndcg,
        });
    }

    let n = per_query.len() as f64;
    RetrievalMetrics {
        mrr: per_query.iter().map(|q| q.reciprocal_rank).sum::<f64>() / n,
        precision_at_k: per_query.iter().map(|q| q.precision_at_k).sum::<f64>() / n,
        recall_at_k: per_query.iter().map(|q| q.recall_at_k).sum::<f64>() / n,
        ndcg_at_k: per_query.iter().map(|q| q.ndcg_at_k).sum::<f64>() / n,
        k,
        query_count: per_query.len(),
        per_query,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reciprocal_rank_first() {
        let ranked = vec!["a".into(), "b".into(), "c".into()];
        let relevant: HashSet<String> = ["a".into()].into();
        assert_eq!(reciprocal_rank(&ranked, &relevant), 1.0);
    }

    #[test]
    fn test_reciprocal_rank_second() {
        let ranked = vec!["b".into(), "a".into(), "c".into()];
        let relevant: HashSet<String> = ["a".into()].into();
        assert_eq!(reciprocal_rank(&ranked, &relevant), 0.5);
    }

    #[test]
    fn test_reciprocal_rank_not_found() {
        let ranked = vec!["b".into(), "c".into()];
        let relevant: HashSet<String> = ["a".into()].into();
        assert_eq!(reciprocal_rank(&ranked, &relevant), 0.0);
    }

    #[test]
    fn test_precision_at_k() {
        let ranked = vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
        let relevant: HashSet<String> = ["a".into(), "c".into(), "e".into()].into();
        assert!((precision_at_k(&ranked, &relevant, 5) - 0.6).abs() < 1e-10);
        assert!((precision_at_k(&ranked, &relevant, 2) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_recall_at_k() {
        let ranked = vec!["a".into(), "b".into(), "c".into()];
        let relevant: HashSet<String> = ["a".into(), "c".into(), "f".into()].into();
        assert!((recall_at_k(&ranked, &relevant, 3) - 2.0 / 3.0).abs() < 1e-10);
        assert!((recall_at_k(&ranked, &relevant, 1) - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_ndcg_perfect_ranking() {
        let ranked = vec![2, 1, 0];
        let ideal = vec![2, 1, 0];
        assert!((ndcg_at_k(&ranked, &ideal, 3) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ndcg_imperfect_ranking() {
        let ranked = vec![0, 2, 1];
        let ideal = vec![2, 1, 0];
        let ndcg = ndcg_at_k(&ranked, &ideal, 3);
        assert!(ndcg < 1.0, "imperfect ranking should have nDCG < 1.0");
        assert!(ndcg > 0.0, "non-empty results should have nDCG > 0.0");
    }

    #[test]
    fn test_dcg_known_values() {
        // DCG for [2, 0, 1] at k=3:
        // (2^2 - 1)/log2(2) + (2^0 - 1)/log2(3) + (2^1 - 1)/log2(4)
        // = 3/1 + 0/1.585 + 1/2
        // = 3.0 + 0.0 + 0.5 = 3.5
        let relevance = vec![2, 0, 1];
        let result = dcg(&relevance, 3);
        assert!((result - 3.5).abs() < 1e-10);
    }
}
