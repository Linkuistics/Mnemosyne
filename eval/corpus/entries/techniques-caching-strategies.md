---
title: Application-Level Caching Patterns
tags: [caching, performance, patterns]
created: 2025-10-18
last_validated: 2026-02-08
confidence: medium
origins:
  - project: product-catalog
    date: 2025-10-18
    context: "Implementing cache-aside pattern for frequently queried categories"
supersedes: []
---

Cache-aside (lazy loading) is the safest default: read from cache, on miss read from source and populate cache. Write-through adds latency to every write but guarantees cache freshness.

Set TTLs based on how stale the data can be, not on how often it changes. A product catalog that changes hourly can tolerate 5-minute staleness; a user permissions cache cannot tolerate any.

Add jitter to TTL values (e.g., base TTL +/- 10%) to prevent thundering herd when many cache entries expire simultaneously.
