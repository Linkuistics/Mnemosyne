---
title: Terraform Module Composition
tags: [terraform, infrastructure, modules]
created: 2025-12-15
last_validated: 2026-02-05
confidence: medium
origins:
  - project: cloud-infrastructure
    date: 2025-12-15
    context: "Refactoring monolithic Terraform config into composable modules"
supersedes: []
---

Keep modules small and single-purpose: one module per logical resource group (e.g., "vpc", "rds-cluster", "ecs-service"). Monolithic modules that provision an entire environment become untestable and have a blast radius equal to the whole stack.

Pin module versions explicitly (`source = "git::...?ref=v1.2.3"`) rather than tracking a branch. An untested module change propagating to all environments simultaneously is a common outage cause.

Use `terraform plan` output as a PR review artifact. If the plan shows unexpected resource deletions or recreations, the change is not safe to apply — regardless of what the diff looks like.
