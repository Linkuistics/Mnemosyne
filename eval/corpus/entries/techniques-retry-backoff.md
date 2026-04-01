---
title: Retry with Exponential Backoff
tags: [retry, backoff, resilience, distributed]
created: 2026-01-05
last_validated: 2026-01-05
confidence: low
origins:
  - project: notification-service
    date: 2026-01-05
    context: "Rate-limited by SMS provider due to aggressive retry loop"
supersedes: []
---

Use exponential backoff with jitter for all retries against external services. Linear or fixed-interval retries amplify load during partial outages — every client retries at the same interval, creating synchronized spikes.

Cap the maximum retry count (3-5) and the maximum backoff interval (30-60 seconds). Unbounded retries with unbounded backoff can keep connections open for minutes, exhausting local resources.

Distinguish between retryable errors (429, 503, connection timeout) and permanent errors (400, 404, authentication failure). Retrying a permanent error wastes time and budget.
