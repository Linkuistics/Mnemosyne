---
title: Monad Transformer Stack Design
tags: [haskell, monads, functional, type-system]
created: 2025-05-20
last_validated: 2026-03-01
confidence: high
origins:
  - project: proof-checker
    date: 2025-05-20
    context: "Designing error handling for multi-phase type checker"
  - project: dsl-compiler
    date: 2026-03-01
    context: "Refactoring interpreter to use ReaderT pattern"
supersedes: []
---

Use the ReaderT design pattern: stack `ReaderT Env IO` as the base, with errors handled via `ExceptT` or `throwIO`. This avoids the quadratic instance problem of deep transformer stacks.

Order matters: `ExceptT e (StateT s IO)` loses state on error, while `StateT s (ExceptT e IO)` preserves it. Choose based on whether partial state should survive failures.

For large applications, define a custom monad with `newtype AppM a = AppM (ReaderT AppEnv IO a)` and derive instances via `GeneralizedNewtypeDeriving`. This provides a stable interface even when the internal stack changes.
