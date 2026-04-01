---
title: Connection Pool Sizing Strategy
tags: [database, pooling, performance, connections]
created: 2025-07-10
last_validated: 2026-01-15
confidence: high
origins:
  - project: order-service
    date: 2025-07-10
    context: "Debugging connection exhaustion under load"
  - project: analytics-api
    date: 2026-01-15
    context: "Load testing revealed pool too small for concurrent queries"
supersedes: []
---

Set connection pool size to approximately 2x the number of CPU cores, not to the database's max_connections. A pool too large causes context switching overhead and lock contention on the database side.

Always configure an idle timeout (30-60 seconds) to reclaim leaked connections. Without this, a slow connection leak can exhaust the pool over hours without any visible error spike.

Monitor pool wait time as a key metric. If requests routinely wait more than 50ms for a connection, the pool is too small — but first check whether queries are holding connections too long.
