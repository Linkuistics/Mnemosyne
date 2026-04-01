---
title: Event Sourcing for Audit-Critical Domains
tags: [event-sourcing, architecture, patterns]
created: 2025-12-20
last_validated: 2025-12-20
confidence: low
origins:
  - project: billing-service
    date: 2025-12-20
    context: "Investigating event sourcing for billing audit trail requirements"
supersedes: []
---

Event sourcing stores state changes as an append-only log of events rather than mutable records. This provides a complete audit trail and enables temporal queries ("what was the state at time T") without additional instrumentation.

The complexity cost is significant: projections must be maintained for every read pattern, event schema evolution requires careful versioning, and debugging requires replaying event sequences rather than inspecting current state.

Only adopt event sourcing when the audit trail or temporal query capability is a hard requirement. For most CRUD applications, a simple `updated_at` column and soft deletes provide sufficient history.
