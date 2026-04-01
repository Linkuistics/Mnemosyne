---
title: ML Data Pipeline Reproducibility
tags: [machine-learning, data, pipelines, python, pandas, numpy]
created: 2025-11-05
last_validated: 2026-01-25
confidence: medium
origins:
  - project: recommendation-engine
    date: 2025-11-05
    context: "Model performance regression traced to undocumented feature engineering change"
supersedes: []
---

Pin every transformation step to a versioned function with frozen random seeds. A pipeline that produces different features from the same raw data on different runs makes debugging regressions nearly impossible.

Store intermediate dataframes with row counts and column checksums at each pipeline stage. When output quality degrades, binary search the stages to find where the data first diverges from the known-good baseline.

Use `pandas.DataFrame.pipe()` to chain transformations rather than reassigning variables. This makes the pipeline order explicit and each step independently testable.
