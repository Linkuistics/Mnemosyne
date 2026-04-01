---
title: Integration Tests Over Mocks for Database Code
tags: [testing, databases, integration]
created: 2025-09-12
last_validated: 2026-01-28
confidence: medium
origins:
  - project: order-service
    date: 2025-09-12
    context: "Mocked tests passed but production migration broke due to column type mismatch"
supersedes: []
---

Run database tests against a real database instance (Docker container or testcontainers). Mocking the database layer creates a false confidence gap — the mock passes but the real query fails on type coercions, NULL handling, or transaction semantics.

Use per-test transactions that roll back after each test for isolation. This is faster than recreating the schema and avoids test pollution without the fragility of manual cleanup.

Write one integration test per query pattern, not per function. If three functions all do `SELECT ... WHERE id = $1`, one test covering that pattern is sufficient.
