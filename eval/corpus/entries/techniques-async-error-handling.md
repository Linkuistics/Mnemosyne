---
title: Error Handling in Async Pipelines
tags: [error-handling, patterns, async]
created: 2025-08-25
last_validated: 2026-03-15
confidence: high
origins:
  - project: message-broker
    date: 2025-08-25
    context: "Unhandled rejections in async message handlers causing silent data loss"
  - project: notification-service
    date: 2026-03-15
    context: "Implementing dead letter queue for failed async operations"
supersedes: []
---

Every async pipeline stage must have an explicit error path. An unhandled error in a fire-and-forget task is silently swallowed in most runtimes (tokio, asyncio, Node.js). At minimum, log the error; ideally, route to a dead letter queue.

Use typed error enums at pipeline stage boundaries rather than string errors. This lets downstream stages distinguish between retryable failures (timeout, rate limit) and permanent failures (validation error, not found).

For fan-out operations (processing N items concurrently), collect errors separately and report them after all items complete. Failing fast on the first error loses the results of already-completed successful items.
