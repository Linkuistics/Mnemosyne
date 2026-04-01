---
title: Legacy Database Migration Lessons
tags: [migration, database, python, legacy]
created: 2025-11-25
last_validated: 2025-11-25
confidence: low
origins:
  - project: crm-migration
    date: 2025-11-25
    context: "Migrating 8-year-old MySQL database to PostgreSQL"
supersedes: []
---

Run the old and new systems in parallel with a comparison layer before cutting over. Shadow-writing to both databases and diffing the results catches data transformation bugs that unit tests miss.

Legacy schemas often have implicit constraints enforced by application code, not database constraints. Catalog these by reading the old application's write paths before designing the new schema — a clean schema that violates implicit constraints will corrupt data.

Migrate in batches with checkpoints, not as a single transaction. A 50GB table migration that fails at 90% and rolls back wastes hours and locks tables.
