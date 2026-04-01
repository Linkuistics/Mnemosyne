---
title: "Rust Async Cancellation: Revised Approach"
tags: [rust, async, tokio, cancellation]
created: 2026-02-15
last_validated: 2026-03-20
confidence: medium
origins:
  - project: api-gateway
    date: 2026-02-15
    context: "Discovered CancellationToken is more reliable than broadcast channels for shutdown"
supersedes: [languages-rust-async.md]
---

Prefer `tokio_util::sync::CancellationToken` over broadcast channels for shutdown coordination. The token propagates through child tasks automatically, whereas broadcast receivers must be manually passed and can be dropped.

For select! branches, always put the cancellation check first. Tokio's select! is biased toward earlier branches when multiple are ready simultaneously, so cancellation takes priority over new work.

Drop guards (`scopeguard`) are more reliable than manual cleanup in cancelled tasks. The previous recommendation to use atomic blocks is fragile — drop guards run regardless of cancellation point.
