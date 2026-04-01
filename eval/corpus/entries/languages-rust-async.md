---
title: Rust Async Task Cancellation
tags: [rust, async, tokio, concurrency]
created: 2025-06-10
last_validated: 2026-01-05
confidence: high
origins:
  - project: message-broker
    date: 2025-06-10
    context: "Debugging dropped futures causing resource leaks"
  - project: api-gateway
    date: 2026-01-05
    context: "Graceful shutdown with in-flight request draining"
supersedes: []
---

Dropping a tokio task cancels it at the next `.await` point, but any state before that point is lost without cleanup. Use `tokio::select!` with a cancellation token for cooperative cancellation that runs destructors.

Always wrap shared state mutations in a block that completes atomically relative to cancellation. A half-applied state change from a cancelled task is a source of subtle corruption bugs.

For graceful shutdown, use `tokio::signal` combined with a broadcast channel to notify all tasks. Each task should check the shutdown signal at natural pause points rather than being forcibly aborted.
