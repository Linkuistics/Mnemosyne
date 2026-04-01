---
title: JWT Token Security Practices
tags: [security, authentication, tokens, api]
created: 2025-04-15
last_validated: 2026-03-01
confidence: high
origins:
  - project: identity-service
    date: 2025-04-15
    context: "Security audit flagged long-lived JWTs without rotation"
  - project: api-gateway
    date: 2026-03-01
    context: "Implementing token refresh flow with sliding window"
supersedes: []
---

Keep JWT expiry short (15 minutes) and use refresh tokens for session continuity. A leaked access token with a 24-hour TTL gives an attacker a full day of access with no revocation mechanism.

Store refresh tokens server-side (database or Redis) so they can be explicitly revoked. JWTs are stateless by design — revocation requires a complementary stateful mechanism.

Never store secrets or PII in JWT claims. The payload is base64-encoded, not encrypted. Anyone with the token can read the claims, even without the signing key.
