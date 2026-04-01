---
title: APIanyware Async FFI Bridge Patterns
tags: [rust, async, ffi, racket, tokio]
created: 2025-08-01
last_validated: 2026-03-25
confidence: high
origins:
  - project: apianyware-macos
    date: 2025-08-01
    context: "Bridging Racket's green threads with tokio runtime via FFI callbacks"
supersedes: []
---

When bridging async runtimes across FFI (Racket to Rust via tokio), use a dedicated tokio runtime on the Rust side rather than sharing the caller's event loop. The FFI boundary creates a thread-safety boundary that shared runtimes cannot respect.

Pass completion callbacks as C function pointers with a `void*` context parameter. The Rust side wraps these in a `oneshot::channel` future so async Rust code can `.await` the FFI result naturally.

Always pin the tokio runtime to a background thread started during library initialization. Creating and destroying runtimes per-call leaks threads and corrupts the reactor's internal state.
