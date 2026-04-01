---
title: Docker Layer Caching Optimization
tags: [docker, optimization]
created: 2025-05-15
last_validated: 2026-03-05
confidence: high
origins:
  - project: ci-infrastructure
    date: 2025-05-15
    context: "Build times reduced from 12min to 3min by reordering COPY instructions"
  - project: api-gateway
    date: 2026-03-05
    context: "Multi-stage builds cut image size from 1.2GB to 180MB"
supersedes: []
---

Order Dockerfile instructions from least-changing to most-changing. `COPY package.json` before `COPY src/` ensures dependency installation is cached when only source code changes.

Use multi-stage builds to separate build dependencies from runtime. The final stage should use a minimal base image (distroless or alpine) and copy only the compiled binary and runtime assets.
