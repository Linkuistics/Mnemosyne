---
title: PostgreSQL Index Selection
tags: [postgres, database, indexing, performance]
created: 2025-06-20
last_validated: 2026-02-25
confidence: high
origins:
  - project: order-service
    date: 2025-06-20
    context: "Query planner ignoring index due to low selectivity on status column"
  - project: analytics-api
    date: 2026-02-25
    context: "Partial index on active records reduced index size by 90%"
supersedes: []
---

Use partial indexes (`WHERE active = true`) for columns with skewed distributions. A full index on a boolean column with 95% true values wastes space and is ignored by the planner anyway.

Composite indexes should order columns by selectivity (most selective first) for equality conditions, and put range conditions last. `CREATE INDEX ON orders (customer_id, created_at)` is far more useful than the reverse.

Always run `EXPLAIN (ANALYZE, BUFFERS)` on production-representative data before and after adding an index. The planner's behavior depends on table statistics, not just schema — a query that uses the index on test data may not use it on production data.
