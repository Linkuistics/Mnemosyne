---
title: Database Connection Lifecycle Management
tags: [database, connections, lifecycle, pooling]
created: 2025-08-18
last_validated: 2026-03-10
confidence: high
origins:
  - project: order-service
    date: 2025-08-18
    context: "Investigating connection leak causing pool exhaustion under load"
  - project: reporting-api
    date: 2026-03-10
    context: "Profiling revealed idle connections consuming database memory"
supersedes: []
---

Always release connections back to the pool in a `finally` block or RAII guard. Connections held across `await` points in async code are a common leak vector — the task can be cancelled before cleanup runs.

Set `idle_timeout` to 5-10 minutes, not unlimited. Long-lived idle connections consume database-side memory and can become stale if the database restarts or a firewall drops the session.

Monitor `active_connections` and `wait_count` as separate metrics. High active count means queries are slow; high wait count means the pool is too small. Different root causes require different fixes.
