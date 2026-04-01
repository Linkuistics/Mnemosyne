use serde::Serialize;

use crate::contradiction::ContradictionMetrics;
use crate::context::ContextMetrics;
use crate::retrieval::RetrievalMetrics;

#[derive(Serialize)]
struct JsonReport {
    retrieval: JsonRetrieval,
    contradiction: JsonContradiction,
    context_detection: JsonContext,
}

#[derive(Serialize)]
struct JsonRetrieval {
    mrr: f64,
    precision_at_k: f64,
    recall_at_k: f64,
    ndcg_at_k: f64,
    k: usize,
    query_count: usize,
}

#[derive(Serialize)]
struct JsonContradiction {
    threshold: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    pair_count: usize,
    sweep: Option<Vec<JsonSweepEntry>>,
}

#[derive(Serialize)]
struct JsonSweepEntry {
    threshold: f64,
    precision: f64,
    recall: f64,
    f1: f64,
}

#[derive(Serialize)]
struct JsonContext {
    language_accuracy: f64,
    dependency_accuracy: f64,
    tag_mapping_accuracy: f64,
    project_count: usize,
}

pub fn format_human(
    retrieval: &RetrievalMetrics,
    contradiction: &ContradictionMetrics,
    context: &ContextMetrics,
    sweep: &Option<Vec<ContradictionMetrics>>,
    verbose: bool,
) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Retrieval Metrics (k={}, {} queries):\n",
        retrieval.k, retrieval.query_count
    ));
    out.push_str(&format!("  MRR:           {:.3}\n", retrieval.mrr));
    out.push_str(&format!("  Precision@{}:   {:.3}\n", retrieval.k, retrieval.precision_at_k));
    out.push_str(&format!("  Recall@{}:      {:.3}\n", retrieval.k, retrieval.recall_at_k));
    out.push_str(&format!("  nDCG@{}:        {:.3}\n", retrieval.k, retrieval.ndcg_at_k));

    if verbose {
        out.push_str("\n  Per-query breakdown:\n");
        for q in &retrieval.per_query {
            out.push_str(&format!(
                "    {}: MRR={:.3}  P@k={:.3}  R@k={:.3}  nDCG={:.3}\n",
                q.query_id, q.reciprocal_rank, q.precision_at_k, q.recall_at_k, q.ndcg_at_k
            ));
        }
    }

    out.push_str(&format!(
        "\nContradiction Detection (threshold={:.2}, {} pairs):\n",
        contradiction.threshold, contradiction.pair_count
    ));
    out.push_str(&format!("  Precision:     {:.3}\n", contradiction.precision));
    out.push_str(&format!("  Recall:        {:.3}\n", contradiction.recall));
    out.push_str(&format!("  F1:            {:.3}\n", contradiction.f1));

    if let Some(sweep) = sweep {
        out.push_str("\n  Threshold Sweep:\n");
        for s in sweep {
            let marker = if (s.threshold - contradiction.threshold).abs() < 0.01 {
                " <-- current default"
            } else {
                ""
            };
            out.push_str(&format!(
                "    {:.2}  P={:.2}  R={:.2}  F1={:.2}{}\n",
                s.threshold, s.precision, s.recall, s.f1, marker
            ));
        }
    }

    out.push_str(&format!(
        "\nContext Detection ({} projects):\n",
        context.project_count
    ));
    out.push_str(&format!("  Language accuracy:    {:.3}\n", context.language_accuracy));
    out.push_str(&format!("  Dependency accuracy:  {:.3}\n", context.dependency_accuracy));
    out.push_str(&format!("  Tag mapping accuracy: {:.3}\n", context.tag_mapping_accuracy));

    out
}

pub fn format_json(
    retrieval: &RetrievalMetrics,
    contradiction: &ContradictionMetrics,
    context: &ContextMetrics,
    sweep: &Option<Vec<ContradictionMetrics>>,
) -> String {
    let report = JsonReport {
        retrieval: JsonRetrieval {
            mrr: retrieval.mrr,
            precision_at_k: retrieval.precision_at_k,
            recall_at_k: retrieval.recall_at_k,
            ndcg_at_k: retrieval.ndcg_at_k,
            k: retrieval.k,
            query_count: retrieval.query_count,
        },
        contradiction: JsonContradiction {
            threshold: contradiction.threshold,
            precision: contradiction.precision,
            recall: contradiction.recall,
            f1: contradiction.f1,
            pair_count: contradiction.pair_count,
            sweep: sweep.as_ref().map(|s| {
                s.iter()
                    .map(|m| JsonSweepEntry {
                        threshold: m.threshold,
                        precision: m.precision,
                        recall: m.recall,
                        f1: m.f1,
                    })
                    .collect()
            }),
        },
        context_detection: JsonContext {
            language_accuracy: context.language_accuracy,
            dependency_accuracy: context.dependency_accuracy,
            tag_mapping_accuracy: context.tag_mapping_accuracy,
            project_count: context.project_count,
        },
    };
    serde_json::to_string_pretty(&report).expect("JSON serialization should not fail")
}
