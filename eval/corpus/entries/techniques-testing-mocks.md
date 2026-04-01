---
title: Mock Database Dependencies for Fast Tests
tags: [testing, databases, mocking]
created: 2025-11-15
last_validated: 2026-02-20
confidence: medium
origins:
  - project: checkout-service
    date: 2025-11-15
    context: "Integration test suite taking 4 minutes, slowing CI feedback loop"
supersedes: []
---

Mock the database layer for unit tests that verify business logic. The logic of "if inventory < quantity, reject order" does not need a real database — testing it with a mock is 100x faster and equally reliable for that specific assertion.

Use interface-based mocking: define a `Repository` trait/interface, implement it with both a real database client and an in-memory mock. This forces clean separation between business logic and data access.

Reserve integration tests for query correctness and schema compatibility. Unit tests with mocks handle everything else — validation rules, state machines, calculation logic.
