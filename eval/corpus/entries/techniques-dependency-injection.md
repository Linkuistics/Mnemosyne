---
title: Dependency Injection for Testability
tags: [dependency-injection, testing, architecture]
created: 2026-03-05
last_validated: 2026-03-05
confidence: prospective
source: horizon-scan
supersedes: []
---

Constructor injection (passing dependencies as parameters) is simpler and more explicit than framework-based DI containers. For Rust, passing trait objects or generic type parameters achieves the same goal without runtime overhead.

Worth investigating whether the current codebase would benefit from extracting I/O boundaries (file system, network, clock) behind traits. This would enable deterministic testing of time-dependent logic without sleep-based tests.
