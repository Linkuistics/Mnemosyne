---
title: Cargo Workspace Organization
tags: [cargo, rust, workspaces]
created: 2025-11-10
last_validated: 2026-01-30
confidence: medium
origins:
  - project: compiler-tools
    date: 2025-11-10
    context: "Splitting monolith crate into workspace for faster incremental builds"
supersedes: []
---

Use a Cargo workspace when a project has 3+ crates or when incremental build times exceed 30 seconds. The shared target directory eliminates duplicate dependency compilation across crates.

Place shared types and traits in a `-core` or `-types` crate with minimal dependencies. This crate should compile in under 5 seconds — if it grows slow, it is accumulating responsibilities that belong elsewhere.

Use `[workspace.dependencies]` to centralize version pins. Inconsistent dependency versions across workspace members cause confusing link errors and bloated binaries.
