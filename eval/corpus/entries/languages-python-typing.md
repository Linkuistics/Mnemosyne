---
title: Python Type Annotation Strategies
tags: [python, typing, mypy]
created: 2025-11-20
last_validated: 2026-01-10
confidence: medium
origins:
  - project: analytics-api
    date: 2025-11-20
    context: "Adding mypy strict mode to existing Flask codebase"
supersedes: []
---

Start with `--strict` mode on new code and `--ignore-missing-imports` for third-party libraries lacking stubs. Gradually tighten existing modules rather than converting everything at once.

Use `TypedDict` for dictionary-heavy APIs instead of plain `dict[str, Any]`. This catches key typos and type mismatches at check time without runtime overhead.

For generic container types in function signatures, prefer `Sequence` over `list` and `Mapping` over `dict` when the function only reads. This makes the API more flexible and communicates intent.
