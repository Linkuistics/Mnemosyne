---
title: Microservices Authentication Architecture
tags: [security, authentication, api, microservices]
created: 2026-03-10
last_validated: 2026-03-10
confidence: prospective
source: horizon-scan
supersedes: []
---

For the planned microservices split, an API gateway with centralized JWT validation would avoid duplicating auth logic across services. Each service would receive a validated token with claims extracted by the gateway.

Consider mutual TLS for inter-service communication as a complement to JWT. mTLS authenticates the calling service itself, not just the end user, which prevents lateral movement if one service is compromised.
