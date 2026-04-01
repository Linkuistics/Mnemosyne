---
title: Structured Logging and Distributed Tracing
tags: [observability, logging, tracing, monitoring]
created: 2026-03-15
last_validated: 2026-03-15
confidence: prospective
source: horizon-scan
supersedes: []
---

Structured logging (JSON format with consistent field names) enables automated analysis that free-text logs cannot support. Fields like `trace_id`, `span_id`, `service`, and `duration_ms` should be present on every log line.

OpenTelemetry is converging as the standard for distributed tracing. Instrumenting with OTel from the start avoids the painful migration from vendor-specific SDKs later. Worth investigating for any multi-service architecture.
