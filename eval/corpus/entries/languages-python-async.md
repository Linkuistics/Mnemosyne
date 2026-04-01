---
title: Python Asyncio Event Loop Pitfalls
tags: [python, async, asyncio]
created: 2025-08-05
last_validated: 2025-12-15
confidence: medium
origins:
  - project: notification-service
    date: 2025-08-05
    context: "Debugging deadlock when mixing sync and async code"
supersedes: []
---

Never call `asyncio.run()` from within an already-running event loop. Use `asyncio.create_task()` or `loop.run_in_executor()` for sync-to-async bridging. Nested `asyncio.run()` raises RuntimeError in Python 3.10+.

CPU-bound work in an async handler blocks the entire event loop. Use `loop.run_in_executor(None, func)` to offload to a thread pool, or `ProcessPoolExecutor` for true parallelism.

Unhandled exceptions in fire-and-forget tasks are silently swallowed. Always store task references and either `await` them or attach an exception callback via `task.add_done_callback()`.
