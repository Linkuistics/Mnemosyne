---
title: API Rate Limiting Implementation
tags: [rate-limiting, api, performance]
created: 2025-10-05
last_validated: 2026-02-12
confidence: medium
origins:
  - project: platform-api
    date: 2025-10-05
    context: "Implementing per-tenant rate limits after a single client saturated shared resources"
supersedes: []
---

Use the token bucket algorithm for rate limiting — it naturally allows short bursts while enforcing an average rate. Sliding window counters are simpler but don't handle burst patterns well.

Rate limit by API key or tenant ID, not by IP address. IP-based limits break for clients behind NAT or corporate proxies where thousands of users share one IP.

Return `Retry-After` header with the rate limit response (429). Clients that respect this header will self-throttle; without it, they retry immediately and amplify the load.
