---
title: Rust Lifetime Annotations
tags: [rust, lifetimes, memory, ownership]
created: 2025-09-15
last_validated: 2026-02-20
confidence: high
origins:
  - project: compiler-tools
    date: 2025-09-15
    context: "Implementing custom AST visitor with borrowed references"
  - project: data-pipeline
    date: 2026-02-20
    context: "Refactoring streaming parser to avoid unnecessary clones"
supersedes: []
---

Explicit lifetime annotations are required when a function returns a reference that could derive from multiple input references. The compiler cannot infer which input lifetime the output reference should be tied to.

When a struct holds references, annotate the struct with a lifetime parameter and bind all reference fields to it. This prevents dangling references when the struct outlives the data it borrows.

Avoid using `'static` to suppress lifetime errors. It works but hides the real ownership question and makes the API rigid. Propagate the lifetime parameter to callers instead.
