---
title: Conservative HTTP Caching Defaults
tags: [web, caching, http]
created: 2025-10-12
last_validated: 2025-10-12
confidence: low
origins:
  - project: content-api
    date: 2025-10-12
    context: "Users seeing stale content after publish due to aggressive caching"
supersedes: []
---

Default to short `max-age` values (60-300 seconds) for API responses. Aggressive caching causes stale content bugs that are invisible during development and only manifest in production under specific timing conditions.

Use `Cache-Control: private, no-cache` for authenticated endpoints. Browser caches shared across tabs can leak user-specific data if cache headers are permissive.

Always include `Vary: Accept-Encoding, Authorization` when responses differ by these headers, or reverse proxies will serve wrong cached variants.
