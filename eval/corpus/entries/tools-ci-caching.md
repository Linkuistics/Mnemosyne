---
title: GitHub Actions Cache Configuration
tags: [ci, caching, github-actions]
created: 2026-02-10
last_validated: 2026-02-10
confidence: low
origins:
  - project: open-source-lib
    date: 2026-02-10
    context: "CI builds taking 8 minutes due to uncached dependency downloads"
supersedes: []
---

Cache dependency directories (node_modules, target/, .venv) with a key derived from the lockfile hash. Use `hashFiles('**/Cargo.lock')` not `hashFiles('Cargo.lock')` to catch workspace members.

Set `restore-keys` to a prefix of the primary key so partial cache hits still save time. A stale cache with most dependencies is much faster than a cold start.

Beware cache size limits (10 GB per repo). Large Rust target directories can exceed this. Cache only the registry and git deps, not the full target directory.
