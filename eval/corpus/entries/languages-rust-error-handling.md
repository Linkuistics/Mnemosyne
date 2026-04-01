---
title: Rust Error Handling with thiserror and anyhow
tags: [rust, error-handling, patterns]
created: 2025-07-22
last_validated: 2026-02-28
confidence: high
origins:
  - project: api-gateway
    date: 2025-07-22
    context: "Unifying error types across HTTP, database, and auth layers"
  - project: cli-tools
    date: 2026-02-28
    context: "Improving error messages for user-facing CLI commands"
supersedes: []
---

Use `thiserror` for library error types (structured, matchable by callers) and `anyhow` for application-level error propagation (convenient, context-rich). Mixing both in the same crate is the intended pattern.

Add `.context("what was being attempted")` to every `?` in application code. Without context, a bare IO error like "permission denied" gives no clue which file or operation failed.

Never use `unwrap()` in library code. Use `expect("reason this should never fail")` only when the invariant is documented and genuinely upheld. In application code, prefer `?` with context over `unwrap`.
