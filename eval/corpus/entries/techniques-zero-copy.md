---
title: Zero-Copy Deserialization
tags: [zero-copy, performance, rust, serialization]
created: 2026-03-01
last_validated: 2026-03-01
confidence: prospective
source: horizon-scan
supersedes: []
---

Libraries like `rkyv` and `zerocopy` enable deserializing data without copying it out of the source buffer. This eliminates allocation overhead in the hot path for message parsing or memory-mapped file access.

The trade-off is that the deserialized data borrows from the source buffer, so the buffer must outlive all references. This creates lifetime constraints that may propagate through the API and complicate ownership.

Worth benchmarking against standard serde for the message broker's binary protocol. If message throughput is the bottleneck, zero-copy could be a significant win.
