---
title: Frontend State Management Patterns
tags: [frontend, state-management, react, typescript, redux]
created: 2025-10-30
last_validated: 2026-02-10
confidence: medium
origins:
  - project: admin-dashboard
    date: 2025-10-30
    context: "Refactoring Redux store after prop drilling became unmanageable"
supersedes: []
---

Separate server state (API data) from client state (UI state). Use React Query or SWR for server state — they handle caching, refetching, and staleness. Reserve Redux or Zustand for genuinely client-side state like form drafts and UI preferences.

Colocate state with the component that owns it. Lifting state to a global store "just in case" creates unnecessary re-renders and makes components harder to test in isolation.

For complex forms, `useReducer` with a discriminated union action type is more maintainable than multiple `useState` calls. The reducer centralises validation logic and makes state transitions explicit.
