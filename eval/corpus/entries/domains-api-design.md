---
title: REST API Versioning Strategies
tags: [api, rest, design]
created: 2025-12-01
last_validated: 2026-02-15
confidence: medium
origins:
  - project: platform-api
    date: 2025-12-01
    context: "Breaking change to response format required versioning strategy"
supersedes: []
---

Use URL path versioning (`/v2/resources`) over header-based versioning for public APIs. It is more discoverable, easier to debug, and works with browser testing. Header versioning is appropriate for internal service-to-service APIs.

Version the entire API surface, not individual endpoints. Mixed versioning (some endpoints on v1, others on v2) creates a combinatorial compatibility matrix that is impossible to test exhaustively.

Deprecation: return `Sunset` and `Deprecation` headers on old versions with a concrete shutdown date. Log calls to deprecated versions to identify clients that need migration support.
