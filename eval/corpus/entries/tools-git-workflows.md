---
title: Git Branch Naming and Merge Strategy
tags: [git, branching, workflows]
created: 2025-09-01
last_validated: 2026-01-20
confidence: medium
origins:
  - project: platform-api
    date: 2025-09-01
    context: "Adopting trunk-based development after long-lived branches caused merge hell"
supersedes: []
---

Prefer trunk-based development with short-lived feature branches (1-3 days). Long-lived branches accumulate merge conflicts that grow superlinearly with branch lifetime.

Use squash merges for feature branches to keep main history linear and bisectable. Merge commits are appropriate only for release branches where individual commit provenance matters.

Name branches with a prefix convention: `feat/`, `fix/`, `chore/`. Automated tooling (CI rules, branch protection) can then apply different policies based on prefix.
