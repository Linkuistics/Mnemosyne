---
title: ML Pipeline v2 Refactoring Notes
tags: [machine-learning, pipelines, python, refactoring]
created: 2026-01-15
last_validated: 2026-03-01
confidence: medium
origins:
  - project: recommendation-engine
    date: 2026-01-15
    context: "Refactoring monolithic training script into composable pipeline stages"
supersedes: []
---

Split the monolithic training script into discrete stages: data loading, feature engineering, model training, evaluation. Each stage reads from and writes to a versioned artifact store (e.g., a timestamped directory in S3).

Use a configuration object (dataclass or Pydantic model) rather than CLI arguments for pipeline parameters. This makes the configuration serializable, diffable, and reproducible — critical for experiment tracking.

The original pipeline failed silently when feature columns were missing. Adding schema validation (pandera or great_expectations) at stage boundaries catches data issues before they propagate to training.
