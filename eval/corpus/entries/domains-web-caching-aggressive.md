---
title: Aggressive Client-Side Caching for Static Assets
tags: [web, caching, http, performance]
created: 2026-01-20
last_validated: 2026-01-20
confidence: low
origins:
  - project: marketing-site
    date: 2026-01-20
    context: "Core Web Vitals optimization — LCP improved 40% with immutable caching"
supersedes: []
---

Set `Cache-Control: public, max-age=31536000, immutable` for hashed static assets. A one-year TTL with content hashing eliminates unnecessary revalidation requests entirely.

For API responses that change infrequently, use `stale-while-revalidate=86400` to serve cached data immediately while refreshing in the background. This is almost always better than short max-age values that cause visible loading delays.

Aggressive caching is safe when combined with cache-busting URLs (content hashes in filenames). The common advice to use short TTLs is overly conservative for modern deployment pipelines.
