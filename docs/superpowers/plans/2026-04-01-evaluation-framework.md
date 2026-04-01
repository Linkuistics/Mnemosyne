# Evaluation Framework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build two evaluation harnesses — a Rust binary for retrieval/contradiction/context metrics, and a Python package for LLM-judged quality — sharing a benchmark corpus of 39 synthetic knowledge entries.

**Architecture:** The eval crate is a standalone Rust binary depending on the `mnemosyne` library via path. It loads a benchmark corpus (entries + queries + contradiction pairs + mock projects), runs the library's search/detection functions, and reports quantitative metrics (MRR, nDCG, F1, accuracy). The Python quality package uses the Anthropic SDK to score entries against a rubric via LLM-as-judge with variance reduction. Both harnesses are invoked manually, not integrated into CI.

**Tech Stack:** Rust (clap, serde, serde_yaml, serde_json), Python 3.10+ (anthropic, pyyaml)

---

## File Structure

### Shared Corpus

- `eval/corpus/entries/*.md` — 39 synthetic knowledge entries (YAML frontmatter + Markdown body)
- `eval/corpus/queries.yaml` — 20 queries with graded relevance judgements
- `eval/corpus/contradictions.yaml` — 8 contradiction pairs with labels
- `eval/corpus/projects/rust-web-api/` — Mock project for context detection
- `eval/corpus/projects/python-ml-pipeline/` — Mock project for context detection
- `eval/corpus/projects/haskell-web-app/` — Mock project for context detection
- `eval/corpus/projects/rust-python-mixed/` — Multi-language mock project

### Rust Harness

- `eval/Cargo.toml` — Crate manifest, depends on `mnemosyne = { path = ".." }`
- `eval/src/main.rs` — CLI entry point (clap), dispatches to evaluation modules
- `eval/src/corpus.rs` — Load entries, queries, contradictions, and project fixtures from disk
- `eval/src/retrieval.rs` — MRR, precision@k, recall@k, nDCG@k computation
- `eval/src/contradiction.rs` — F1, precision, recall for contradiction detection; threshold sweep
- `eval/src/context.rs` — Language, dependency, and tag mapping accuracy
- `eval/src/report.rs` — Human-readable and JSON output formatting

### Python Quality Harness

- `eval/quality/pyproject.toml` — Package metadata and dependencies
- `eval/quality/rubrics/entry_quality.yaml` — Rubric with 4 dimensions (specificity, actionability, provenance, confidence fit)
- `eval/quality/src/__init__.py` — Package init
- `eval/quality/src/__main__.py` — CLI entry point
- `eval/quality/src/judge.py` — Provider-agnostic Judge protocol
- `eval/quality/src/providers/__init__.py` — Providers package
- `eval/quality/src/providers/claude.py` — Anthropic SDK implementation
- `eval/quality/src/rubric.py` — Load and format rubric YAML into prompts
- `eval/quality/src/structural.py` — Automated structural completeness checks
- `eval/quality/src/report.py` — Aggregate scores, format output
- `eval/quality/src/config.py` — Provider selection, API key handling

---

## Corpus Design

### Coverage Matrix

39 entries across 5 axes, 4 confidence levels, tag densities from 1 to 6.

| # | Filename | Axis | Tags | Confidence | Origins |
|---|----------|------|------|------------|---------|
| 1 | languages-rust-lifetimes.md | languages | rust, lifetimes, memory, ownership | high | 2 projects |
| 2 | languages-rust-async.md | languages | rust, async, tokio, concurrency | high | 2 projects |
| 3 | languages-rust-async-updated.md | languages | rust, async, tokio, cancellation | medium | 1 project |
| 4 | languages-python-typing.md | languages | python, typing, mypy | medium | 1 project |
| 5 | languages-python-async.md | languages | python, async, asyncio | medium | 1 project |
| 6 | languages-typescript-generics.md | languages | typescript, generics, type-system | low | 1 project |
| 7 | languages-haskell-monads.md | languages | haskell, monads, functional, type-system | high | 2 projects |
| 8 | languages-rust-error-handling.md | languages | rust, error-handling, patterns | high | 2 projects |
| 9 | domains-web-caching.md | domains | web, caching, http | low | 1 project |
| 10 | domains-web-caching-aggressive.md | domains | web, caching, http, performance | low | 1 project |
| 11 | domains-database-connections.md | domains | database, connections, lifecycle, pooling | high | 2 projects |
| 12 | domains-api-design.md | domains | api, rest, design | medium | 1 project |
| 13 | domains-ml-data-pipelines.md | domains | machine-learning, data, pipelines, python, pandas, numpy | medium | 1 project |
| 14 | domains-security-auth.md | domains | security, authentication, tokens, api | high | 2 projects |
| 15 | domains-distributed-systems.md | domains | distributed, consensus, networking | low | 1 project |
| 16 | domains-frontend-state.md | domains | frontend, state-management, react, typescript, redux | medium | 1 project |
| 17 | domains-observability.md | domains | observability, logging, tracing, monitoring | prospective | 0 (source: horizon-scan) |
| 18 | tools-docker-layers.md | tools | docker, optimization | high | 2 projects |
| 19 | tools-git-workflows.md | tools | git, branching, workflows | medium | 1 project |
| 20 | tools-ci-caching.md | tools | ci, caching, github-actions | low | 1 project |
| 21 | tools-postgres-indexing.md | tools | postgres, database, indexing, performance | high | 2 projects |
| 22 | tools-vscode-debugging.md | tools | debugging | prospective | 0 (source: horizon-scan) |
| 23 | tools-terraform-modules.md | tools | terraform, infrastructure, modules | medium | 1 project |
| 24 | tools-cargo-workspaces.md | tools | cargo, rust, workspaces | medium | 1 project |
| 25 | techniques-connection-pooling.md | techniques | database, pooling, performance, connections | high | 2 projects |
| 26 | techniques-async-error-handling.md | techniques | error-handling, patterns, async | high | 2 projects |
| 27 | techniques-caching-strategies.md | techniques | caching, performance, patterns | medium | 1 project |
| 28 | techniques-testing-integration.md | techniques | testing, databases, integration | medium | 1 project |
| 29 | techniques-testing-mocks.md | techniques | testing, databases, mocking | medium | 1 project |
| 30 | techniques-retry-backoff.md | techniques | retry, backoff, resilience, distributed | low | 1 project |
| 31 | techniques-dependency-injection.md | techniques | dependency-injection, testing, architecture | prospective | 0 (source: horizon-scan) |
| 32 | techniques-event-sourcing.md | techniques | event-sourcing, architecture, patterns | low | 1 project |
| 33 | techniques-rate-limiting.md | techniques | rate-limiting, api, performance | medium | 1 project |
| 34 | techniques-zero-copy.md | techniques | zero-copy, performance, rust, serialization | prospective | 0 (source: horizon-scan) |
| 35 | projects-apianyware-async.md | projects | rust, async, ffi, racket, tokio | high | 1 project |
| 36 | projects-legacy-migration.md | projects | migration, database, python, legacy | low | 1 project |
| 37 | projects-ml-pipeline-v2.md | projects | machine-learning, pipelines, python, refactoring | medium | 1 project |
| 38 | projects-react-dashboard.md | projects | react, typescript, dashboard, frontend | prospective | 0 (source: horizon-scan) |
| 39 | projects-microservices-auth.md | projects | security, authentication, api, microservices | prospective | 0 (source: horizon-scan) |

**Distribution:** high=11, medium=13, low=8, prospective=7. Tag density: 1–6.

### Contradiction Pairs Design

Jaccard similarity determines detection at threshold 0.5. Pairs designed to produce a meaningful precision-recall sweep:

| Pair | Entry A | Entry B | Jaccard | Label | Note |
|------|---------|---------|---------|-------|------|
| 1 | languages-rust-async.md | languages-rust-async-updated.md | 0.60 | TRUE | Updated cancellation semantics supersede original |
| 2 | domains-web-caching.md | domains-web-caching-aggressive.md | 0.75 | TRUE | Contradictory caching TTL recommendations |
| 3 | techniques-testing-integration.md | techniques-testing-mocks.md | 0.50 | TRUE | Opposing stances on mocking database dependencies |
| 4 | techniques-connection-pooling.md | domains-database-connections.md | 0.60 | FALSE | Complementary: pool sizing vs connection lifecycle |
| 5 | languages-rust-error-handling.md | techniques-async-error-handling.md | 0.50 | FALSE | Different scope: Rust-specific vs async patterns |
| 6 | domains-security-auth.md | projects-microservices-auth.md | 0.60 | FALSE | General auth security vs project-specific architecture |
| 7 | languages-python-async.md | languages-python-typing.md | 0.20 | FALSE | Near-miss: both Python, completely different topics |
| 8 | techniques-caching-strategies.md | tools-ci-caching.md | 0.20 | FALSE | Near-miss: "caching" tag shared, different domains |

**Expected at default threshold 0.5:** 6 flagged (3 TP, 3 FP), P=0.50, R=1.00, F1=0.67.

---

## Phase 1: Corpus + Rust Evaluation Harness

### Task 1: Scaffold eval crate

**Files:**
- Create: `eval/Cargo.toml`
- Create: `eval/src/main.rs`

- [ ] **Step 1: Create eval/Cargo.toml**

```toml
[package]
name = "mnemosyne-eval"
version = "0.1.0"
edition = "2021"

[dependencies]
mnemosyne = { path = ".." }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"
anyhow = "1"
```

- [ ] **Step 2: Create eval/src/main.rs skeleton**

```rust
use std::path::PathBuf;
use clap::Parser;

mod corpus;
mod retrieval;
mod contradiction;
mod context;
mod report;

#[derive(Parser)]
#[command(name = "mnemosyne-eval")]
struct Cli {
    /// Path to corpus directory
    #[arg(long, default_value = "corpus")]
    corpus: PathBuf,

    /// Top-k for retrieval metrics
    #[arg(long, default_value = "5")]
    k: usize,

    /// Run contradiction threshold sweep
    #[arg(long)]
    sweep: bool,

    /// Per-query and per-pair breakdown
    #[arg(long)]
    verbose: bool,

    /// Output in JSON format
    #[arg(long)]
    json: bool,
}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    println!("mnemosyne-eval: scaffold OK");
    Ok(())
}
```

- [ ] **Step 3: Create stub module files**

Create empty files so the crate compiles:
- `eval/src/corpus.rs` — empty
- `eval/src/retrieval.rs` — empty
- `eval/src/contradiction.rs` — empty
- `eval/src/context.rs` — empty
- `eval/src/report.rs` — empty

- [ ] **Step 4: Verify it compiles**

Run: `cd eval && cargo build`
Expected: Compiles successfully.

- [ ] **Step 5: Commit**

```bash
git add eval/Cargo.toml eval/src/
git commit -m "feat(eval): scaffold mnemosyne-eval crate with CLI skeleton"
```

---

### Task 2: Create benchmark corpus entries

**Files:**
- Create: `eval/corpus/entries/` — 39 entry files

All entries follow the Mnemosyne knowledge format: YAML frontmatter with `title`, `tags`, `created`, `last_validated`, `confidence`, `origins`, `supersedes`, then a Markdown body. Entries must be realistic developer knowledge. Refer to the coverage matrix above for metadata.

**Important:** Tags must be EXACT — the queries and contradiction pairs in Task 3 depend on them.

- [ ] **Step 1: Create languages axis entries (8 files)**

`eval/corpus/entries/languages-rust-lifetimes.md`:
```markdown
---
title: Rust Lifetime Annotations
tags: [rust, lifetimes, memory, ownership]
created: 2025-09-15
last_validated: 2026-02-20
confidence: high
origins:
  - project: compiler-tools
    date: 2025-09-15
    context: "Implementing custom AST visitor with borrowed references"
  - project: data-pipeline
    date: 2026-02-20
    context: "Refactoring streaming parser to avoid unnecessary clones"
supersedes: []
---

Explicit lifetime annotations are required when a function returns a reference that could derive from multiple input references. The compiler cannot infer which input lifetime the output reference should be tied to.

When a struct holds references, annotate the struct with a lifetime parameter and bind all reference fields to it. This prevents dangling references when the struct outlives the data it borrows.

Avoid using `'static` to suppress lifetime errors. It works but hides the real ownership question and makes the API rigid. Propagate the lifetime parameter to callers instead.
```

`eval/corpus/entries/languages-rust-async.md`:
```markdown
---
title: Rust Async Task Cancellation
tags: [rust, async, tokio, concurrency]
created: 2025-06-10
last_validated: 2026-01-05
confidence: high
origins:
  - project: message-broker
    date: 2025-06-10
    context: "Debugging dropped futures causing resource leaks"
  - project: api-gateway
    date: 2026-01-05
    context: "Graceful shutdown with in-flight request draining"
supersedes: []
---

Dropping a tokio task cancels it at the next `.await` point, but any state before that point is lost without cleanup. Use `tokio::select!` with a cancellation token for cooperative cancellation that runs destructors.

Always wrap shared state mutations in a block that completes atomically relative to cancellation. A half-applied state change from a cancelled task is a source of subtle corruption bugs.

For graceful shutdown, use `tokio::signal` combined with a broadcast channel to notify all tasks. Each task should check the shutdown signal at natural pause points rather than being forcibly aborted.
```

`eval/corpus/entries/languages-rust-async-updated.md`:
```markdown
---
title: "Rust Async Cancellation: Revised Approach"
tags: [rust, async, tokio, cancellation]
created: 2026-02-15
last_validated: 2026-03-20
confidence: medium
origins:
  - project: api-gateway
    date: 2026-02-15
    context: "Discovered CancellationToken is more reliable than broadcast channels for shutdown"
supersedes: [languages-rust-async.md]
---

Prefer `tokio_util::sync::CancellationToken` over broadcast channels for shutdown coordination. The token propagates through child tasks automatically, whereas broadcast receivers must be manually passed and can be dropped.

For select! branches, always put the cancellation check first. Tokio's select! is biased toward earlier branches when multiple are ready simultaneously, so cancellation takes priority over new work.

Drop guards (`scopeguard`) are more reliable than manual cleanup in cancelled tasks. The previous recommendation to use atomic blocks is fragile — drop guards run regardless of cancellation point.
```

`eval/corpus/entries/languages-python-typing.md`:
```markdown
---
title: Python Type Annotation Strategies
tags: [python, typing, mypy]
created: 2025-11-20
last_validated: 2026-01-10
confidence: medium
origins:
  - project: analytics-api
    date: 2025-11-20
    context: "Adding mypy strict mode to existing Flask codebase"
supersedes: []
---

Start with `--strict` mode on new code and `--ignore-missing-imports` for third-party libraries lacking stubs. Gradually tighten existing modules rather than converting everything at once.

Use `TypedDict` for dictionary-heavy APIs instead of plain `dict[str, Any]`. This catches key typos and type mismatches at check time without runtime overhead.

For generic container types in function signatures, prefer `Sequence` over `list` and `Mapping` over `dict` when the function only reads. This makes the API more flexible and communicates intent.
```

`eval/corpus/entries/languages-python-async.md`:
```markdown
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
```

`eval/corpus/entries/languages-typescript-generics.md`:
```markdown
---
title: TypeScript Generic Constraints
tags: [typescript, generics, type-system]
created: 2026-01-08
last_validated: 2026-01-08
confidence: low
origins:
  - project: component-library
    date: 2026-01-08
    context: "Building polymorphic component props with type-safe event handlers"
supersedes: []
---

Use `extends` constraints to bound generics rather than casting. `function merge<T extends object>(a: T, b: Partial<T>): T` is safer than using `any` and casting the result.

Conditional types with `infer` can extract nested types without manual type parameter threading. `type UnwrapPromise<T> = T extends Promise<infer U> ? U : T` eliminates a class of async return type mismatches.

Template literal types combined with mapped types can generate exhaustive union types from string constants, catching typos at compile time.
```

`eval/corpus/entries/languages-haskell-monads.md`:
```markdown
---
title: Monad Transformer Stack Design
tags: [haskell, monads, functional, type-system]
created: 2025-05-20
last_validated: 2026-03-01
confidence: high
origins:
  - project: proof-checker
    date: 2025-05-20
    context: "Designing error handling for multi-phase type checker"
  - project: dsl-compiler
    date: 2026-03-01
    context: "Refactoring interpreter to use ReaderT pattern"
supersedes: []
---

Use the ReaderT design pattern: stack `ReaderT Env IO` as the base, with errors handled via `ExceptT` or `throwIO`. This avoids the quadratic instance problem of deep transformer stacks.

Order matters: `ExceptT e (StateT s IO)` loses state on error, while `StateT s (ExceptT e IO)` preserves it. Choose based on whether partial state should survive failures.

For large applications, define a custom monad with `newtype AppM a = AppM (ReaderT AppEnv IO a)` and derive instances via `GeneralizedNewtypeDeriving`. This provides a stable interface even when the internal stack changes.
```

`eval/corpus/entries/languages-rust-error-handling.md`:
```markdown
---
title: Rust Error Handling with thiserror and anyhow
tags: [rust, error-handling, patterns]
created: 2025-07-22
last_validated: 2026-02-28
confidence: high
origins:
  - project: api-gateway
    date: 2025-07-22
    context: "Unifying error types across HTTP, database, and auth layers"
  - project: cli-tools
    date: 2026-02-28
    context: "Improving error messages for user-facing CLI commands"
supersedes: []
---

Use `thiserror` for library error types (structured, matchable by callers) and `anyhow` for application-level error propagation (convenient, context-rich). Mixing both in the same crate is the intended pattern.

Add `.context("what was being attempted")` to every `?` in application code. Without context, a bare IO error like "permission denied" gives no clue which file or operation failed.

Never use `unwrap()` in library code. Use `expect("reason this should never fail")` only when the invariant is documented and genuinely upheld. In application code, prefer `?` with context over `unwrap`.
```

- [ ] **Step 2: Create domains axis entries (9 files)**

`eval/corpus/entries/domains-web-caching.md`:
```markdown
---
title: Conservative HTTP Caching Defaults
tags: [web, caching, http]
created: 2025-10-12
last_validated: 2025-10-12
confidence: low
origins:
  - project: content-api
    date: 2025-10-12
    context: "Users seeing stale content after publish due to aggressive caching"
supersedes: []
---

Default to short `max-age` values (60-300 seconds) for API responses. Aggressive caching causes stale content bugs that are invisible during development and only manifest in production under specific timing conditions.

Use `Cache-Control: private, no-cache` for authenticated endpoints. Browser caches shared across tabs can leak user-specific data if cache headers are permissive.

Always include `Vary: Accept-Encoding, Authorization` when responses differ by these headers, or reverse proxies will serve wrong cached variants.
```

`eval/corpus/entries/domains-web-caching-aggressive.md`:
```markdown
---
title: Aggressive Client-Side Caching for Static Assets
tags: [web, caching, http, performance]
created: 2026-01-20
last_validated: 2026-01-20
confidence: low
origins:
  - project: marketing-site
    date: 2026-01-20
    context: "Core Web Vitals optimization — LCP improved 40% with immutable caching"
supersedes: []
---

Set `Cache-Control: public, max-age=31536000, immutable` for hashed static assets. A one-year TTL with content hashing eliminates unnecessary revalidation requests entirely.

For API responses that change infrequently, use `stale-while-revalidate=86400` to serve cached data immediately while refreshing in the background. This is almost always better than short max-age values that cause visible loading delays.

Aggressive caching is safe when combined with cache-busting URLs (content hashes in filenames). The common advice to use short TTLs is overly conservative for modern deployment pipelines.
```

`eval/corpus/entries/domains-database-connections.md`:
```markdown
---
title: Database Connection Lifecycle Management
tags: [database, connections, lifecycle, pooling]
created: 2025-08-18
last_validated: 2026-03-10
confidence: high
origins:
  - project: order-service
    date: 2025-08-18
    context: "Investigating connection leak causing pool exhaustion under load"
  - project: reporting-api
    date: 2026-03-10
    context: "Profiling revealed idle connections consuming database memory"
supersedes: []
---

Always release connections back to the pool in a `finally` block or RAII guard. Connections held across `await` points in async code are a common leak vector — the task can be cancelled before cleanup runs.

Set `idle_timeout` to 5-10 minutes, not unlimited. Long-lived idle connections consume database-side memory and can become stale if the database restarts or a firewall drops the session.

Monitor `active_connections` and `wait_count` as separate metrics. High active count means queries are slow; high wait count means the pool is too small. Different root causes require different fixes.
```

`eval/corpus/entries/domains-api-design.md`:
```markdown
---
title: REST API Versioning Strategies
tags: [api, rest, design]
created: 2025-12-01
last_validated: 2026-02-15
confidence: medium
origins:
  - project: platform-api
    date: 2025-12-01
    context: "Breaking change to response format required versioning strategy"
supersedes: []
---

Use URL path versioning (`/v2/resources`) over header-based versioning for public APIs. It is more discoverable, easier to debug, and works with browser testing. Header versioning is appropriate for internal service-to-service APIs.

Version the entire API surface, not individual endpoints. Mixed versioning (some endpoints on v1, others on v2) creates a combinatorial compatibility matrix that is impossible to test exhaustively.

Deprecation: return `Sunset` and `Deprecation` headers on old versions with a concrete shutdown date. Log calls to deprecated versions to identify clients that need migration support.
```

`eval/corpus/entries/domains-ml-data-pipelines.md`:
```markdown
---
title: ML Data Pipeline Reproducibility
tags: [machine-learning, data, pipelines, python, pandas, numpy]
created: 2025-11-05
last_validated: 2026-01-25
confidence: medium
origins:
  - project: recommendation-engine
    date: 2025-11-05
    context: "Model performance regression traced to undocumented feature engineering change"
supersedes: []
---

Pin every transformation step to a versioned function with frozen random seeds. A pipeline that produces different features from the same raw data on different runs makes debugging regressions nearly impossible.

Store intermediate dataframes with row counts and column checksums at each pipeline stage. When output quality degrades, binary search the stages to find where the data first diverges from the known-good baseline.

Use `pandas.DataFrame.pipe()` to chain transformations rather than reassigning variables. This makes the pipeline order explicit and each step independently testable.
```

`eval/corpus/entries/domains-security-auth.md`:
```markdown
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
```

`eval/corpus/entries/domains-distributed-systems.md`:
```markdown
---
title: Distributed Consensus Trade-offs
tags: [distributed, consensus, networking]
created: 2026-02-01
last_validated: 2026-02-01
confidence: low
origins:
  - project: config-service
    date: 2026-02-01
    context: "Evaluating whether to use Raft for config propagation"
supersedes: []
---

Raft is simpler to implement and debug than Paxos but requires a stable leader. In environments with frequent network partitions, leaderless protocols (like EPaxos) avoid the leader-election storm problem.

For configuration propagation specifically, eventual consistency via gossip protocols is usually sufficient and dramatically simpler. Reserve consensus protocols for cases requiring linearizable reads — most services do not.

The CAP theorem is often misapplied. The real question is not "consistency or availability" but "how much inconsistency is tolerable for how long." Quantify the tolerance before choosing a protocol.
```

`eval/corpus/entries/domains-frontend-state.md`:
```markdown
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
```

`eval/corpus/entries/domains-observability.md`:
```markdown
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
```

- [ ] **Step 3: Create tools axis entries (7 files)**

`eval/corpus/entries/tools-docker-layers.md`:
```markdown
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
```

`eval/corpus/entries/tools-git-workflows.md`:
```markdown
---
title: Git Branch Naming and Merge Strategy
tags: [git, branching, workflows]
created: 2025-09-01
last_validated: 2026-01-20
confidence: medium
origins:
  - project: platform-api
    date: 2025-09-01
    context: "Adopting trunk-based development after long-lived branches caused merge hell"
supersedes: []
---

Prefer trunk-based development with short-lived feature branches (1-3 days). Long-lived branches accumulate merge conflicts that grow superlinearly with branch lifetime.

Use squash merges for feature branches to keep main history linear and bisectable. Merge commits are appropriate only for release branches where individual commit provenance matters.

Name branches with a prefix convention: `feat/`, `fix/`, `chore/`. Automated tooling (CI rules, branch protection) can then apply different policies based on prefix.
```

`eval/corpus/entries/tools-ci-caching.md`:
```markdown
---
title: GitHub Actions Cache Configuration
tags: [ci, caching, github-actions]
created: 2026-02-10
last_validated: 2026-02-10
confidence: low
origins:
  - project: open-source-lib
    date: 2026-02-10
    context: "CI builds taking 8 minutes due to uncached dependency downloads"
supersedes: []
---

Cache dependency directories (node_modules, target/, .venv) with a key derived from the lockfile hash. Use `hashFiles('**/Cargo.lock')` not `hashFiles('Cargo.lock')` to catch workspace members.

Set `restore-keys` to a prefix of the primary key so partial cache hits still save time. A stale cache with most dependencies is much faster than a cold start.

Beware cache size limits (10 GB per repo). Large Rust target directories can exceed this. Cache only the registry and git deps, not the full target directory.
```

`eval/corpus/entries/tools-postgres-indexing.md`:
```markdown
---
title: PostgreSQL Index Selection
tags: [postgres, database, indexing, performance]
created: 2025-06-20
last_validated: 2026-02-25
confidence: high
origins:
  - project: order-service
    date: 2025-06-20
    context: "Query planner ignoring index due to low selectivity on status column"
  - project: analytics-api
    date: 2026-02-25
    context: "Partial index on active records reduced index size by 90%"
supersedes: []
---

Use partial indexes (`WHERE active = true`) for columns with skewed distributions. A full index on a boolean column with 95% true values wastes space and is ignored by the planner anyway.

Composite indexes should order columns by selectivity (most selective first) for equality conditions, and put range conditions last. `CREATE INDEX ON orders (customer_id, created_at)` is far more useful than the reverse.

Always run `EXPLAIN (ANALYZE, BUFFERS)` on production-representative data before and after adding an index. The planner's behavior depends on table statistics, not just schema — a query that uses the index on test data may not use it on production data.
```

`eval/corpus/entries/tools-vscode-debugging.md`:
```markdown
---
title: Editor-Based Debugging Setup
tags: [debugging]
created: 2026-03-10
last_validated: 2026-03-10
confidence: prospective
source: horizon-scan
supersedes: []
---

Configuring launch.json for multi-language projects (Rust + Python) might enable setting breakpoints across FFI boundaries. Worth investigating whether DAP supports this workflow or whether separate debug sessions are required.
```

`eval/corpus/entries/tools-terraform-modules.md`:
```markdown
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
```

`eval/corpus/entries/tools-cargo-workspaces.md`:
```markdown
---
title: Cargo Workspace Organization
tags: [cargo, rust, workspaces]
created: 2025-11-10
last_validated: 2026-01-30
confidence: medium
origins:
  - project: compiler-tools
    date: 2025-11-10
    context: "Splitting monolith crate into workspace for faster incremental builds"
supersedes: []
---

Use a Cargo workspace when a project has 3+ crates or when incremental build times exceed 30 seconds. The shared target directory eliminates duplicate dependency compilation across crates.

Place shared types and traits in a `-core` or `-types` crate with minimal dependencies. This crate should compile in under 5 seconds — if it grows slow, it is accumulating responsibilities that belong elsewhere.

Use `[workspace.dependencies]` to centralize version pins. Inconsistent dependency versions across workspace members cause confusing link errors and bloated binaries.
```

- [ ] **Step 4: Create techniques axis entries (10 files)**

`eval/corpus/entries/techniques-connection-pooling.md`:
```markdown
---
title: Connection Pool Sizing Strategy
tags: [database, pooling, performance, connections]
created: 2025-07-10
last_validated: 2026-01-15
confidence: high
origins:
  - project: order-service
    date: 2025-07-10
    context: "Debugging connection exhaustion under load"
  - project: analytics-api
    date: 2026-01-15
    context: "Load testing revealed pool too small for concurrent queries"
supersedes: []
---

Set connection pool size to approximately 2x the number of CPU cores, not to the database's max_connections. A pool too large causes context switching overhead and lock contention on the database side.

Always configure an idle timeout (30-60 seconds) to reclaim leaked connections. Without this, a slow connection leak can exhaust the pool over hours without any visible error spike.

Monitor pool wait time as a key metric. If requests routinely wait more than 50ms for a connection, the pool is too small — but first check whether queries are holding connections too long.
```

`eval/corpus/entries/techniques-async-error-handling.md`:
```markdown
---
title: Error Handling in Async Pipelines
tags: [error-handling, patterns, async]
created: 2025-08-25
last_validated: 2026-03-15
confidence: high
origins:
  - project: message-broker
    date: 2025-08-25
    context: "Unhandled rejections in async message handlers causing silent data loss"
  - project: notification-service
    date: 2026-03-15
    context: "Implementing dead letter queue for failed async operations"
supersedes: []
---

Every async pipeline stage must have an explicit error path. An unhandled error in a fire-and-forget task is silently swallowed in most runtimes (tokio, asyncio, Node.js). At minimum, log the error; ideally, route to a dead letter queue.

Use typed error enums at pipeline stage boundaries rather than string errors. This lets downstream stages distinguish between retryable failures (timeout, rate limit) and permanent failures (validation error, not found).

For fan-out operations (processing N items concurrently), collect errors separately and report them after all items complete. Failing fast on the first error loses the results of already-completed successful items.
```

`eval/corpus/entries/techniques-caching-strategies.md`:
```markdown
---
title: Application-Level Caching Patterns
tags: [caching, performance, patterns]
created: 2025-10-18
last_validated: 2026-02-08
confidence: medium
origins:
  - project: product-catalog
    date: 2025-10-18
    context: "Implementing cache-aside pattern for frequently queried categories"
supersedes: []
---

Cache-aside (lazy loading) is the safest default: read from cache, on miss read from source and populate cache. Write-through adds latency to every write but guarantees cache freshness.

Set TTLs based on how stale the data can be, not on how often it changes. A product catalog that changes hourly can tolerate 5-minute staleness; a user permissions cache cannot tolerate any.

Add jitter to TTL values (e.g., base TTL +/- 10%) to prevent thundering herd when many cache entries expire simultaneously.
```

`eval/corpus/entries/techniques-testing-integration.md`:
```markdown
---
title: Integration Tests Over Mocks for Database Code
tags: [testing, databases, integration]
created: 2025-09-12
last_validated: 2026-01-28
confidence: medium
origins:
  - project: order-service
    date: 2025-09-12
    context: "Mocked tests passed but production migration broke due to column type mismatch"
supersedes: []
---

Run database tests against a real database instance (Docker container or testcontainers). Mocking the database layer creates a false confidence gap — the mock passes but the real query fails on type coercions, NULL handling, or transaction semantics.

Use per-test transactions that roll back after each test for isolation. This is faster than recreating the schema and avoids test pollution without the fragility of manual cleanup.

Write one integration test per query pattern, not per function. If three functions all do `SELECT ... WHERE id = $1`, one test covering that pattern is sufficient.
```

`eval/corpus/entries/techniques-testing-mocks.md`:
```markdown
---
title: Mock Database Dependencies for Fast Tests
tags: [testing, databases, mocking]
created: 2025-11-15
last_validated: 2026-02-20
confidence: medium
origins:
  - project: checkout-service
    date: 2025-11-15
    context: "Integration test suite taking 4 minutes, slowing CI feedback loop"
supersedes: []
---

Mock the database layer for unit tests that verify business logic. The logic of "if inventory < quantity, reject order" does not need a real database — testing it with a mock is 100x faster and equally reliable for that specific assertion.

Use interface-based mocking: define a `Repository` trait/interface, implement it with both a real database client and an in-memory mock. This forces clean separation between business logic and data access.

Reserve integration tests for query correctness and schema compatibility. Unit tests with mocks handle everything else — validation rules, state machines, calculation logic.
```

`eval/corpus/entries/techniques-retry-backoff.md`:
```markdown
---
title: Retry with Exponential Backoff
tags: [retry, backoff, resilience, distributed]
created: 2026-01-05
last_validated: 2026-01-05
confidence: low
origins:
  - project: notification-service
    date: 2026-01-05
    context: "Rate-limited by SMS provider due to aggressive retry loop"
supersedes: []
---

Use exponential backoff with jitter for all retries against external services. Linear or fixed-interval retries amplify load during partial outages — every client retries at the same interval, creating synchronized spikes.

Cap the maximum retry count (3-5) and the maximum backoff interval (30-60 seconds). Unbounded retries with unbounded backoff can keep connections open for minutes, exhausting local resources.

Distinguish between retryable errors (429, 503, connection timeout) and permanent errors (400, 404, authentication failure). Retrying a permanent error wastes time and budget.
```

`eval/corpus/entries/techniques-dependency-injection.md`:
```markdown
---
title: Dependency Injection for Testability
tags: [dependency-injection, testing, architecture]
created: 2026-03-05
last_validated: 2026-03-05
confidence: prospective
source: horizon-scan
supersedes: []
---

Constructor injection (passing dependencies as parameters) is simpler and more explicit than framework-based DI containers. For Rust, passing trait objects or generic type parameters achieves the same goal without runtime overhead.

Worth investigating whether the current codebase would benefit from extracting I/O boundaries (file system, network, clock) behind traits. This would enable deterministic testing of time-dependent logic without sleep-based tests.
```

`eval/corpus/entries/techniques-event-sourcing.md`:
```markdown
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
```

`eval/corpus/entries/techniques-rate-limiting.md`:
```markdown
---
title: API Rate Limiting Implementation
tags: [rate-limiting, api, performance]
created: 2025-10-05
last_validated: 2026-02-12
confidence: medium
origins:
  - project: platform-api
    date: 2025-10-05
    context: "Implementing per-tenant rate limits after a single client saturated shared resources"
supersedes: []
---

Use the token bucket algorithm for rate limiting — it naturally allows short bursts while enforcing an average rate. Sliding window counters are simpler but don't handle burst patterns well.

Rate limit by API key or tenant ID, not by IP address. IP-based limits break for clients behind NAT or corporate proxies where thousands of users share one IP.

Return `Retry-After` header with the rate limit response (429). Clients that respect this header will self-throttle; without it, they retry immediately and amplify the load.
```

`eval/corpus/entries/techniques-zero-copy.md`:
```markdown
---
title: Zero-Copy Deserialization
tags: [zero-copy, performance, rust, serialization]
created: 2026-03-01
last_validated: 2026-03-01
confidence: prospective
source: horizon-scan
supersedes: []
---

Libraries like `rkyv` and `zerocopy` enable deserializing data without copying it out of the source buffer. This eliminates allocation overhead in the hot path for message parsing or memory-mapped file access.

The trade-off is that the deserialized data borrows from the source buffer, so the buffer must outlive all references. This creates lifetime constraints that may propagate through the API and complicate ownership.

Worth benchmarking against standard serde for the message broker's binary protocol. If message throughput is the bottleneck, zero-copy could be a significant win.
```

- [ ] **Step 5: Create projects axis entries (5 files)**

`eval/corpus/entries/projects-apianyware-async.md`:
```markdown
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
```

`eval/corpus/entries/projects-legacy-migration.md`:
```markdown
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
```

`eval/corpus/entries/projects-ml-pipeline-v2.md`:
```markdown
---
title: ML Pipeline v2 Refactoring Notes
tags: [machine-learning, pipelines, python, refactoring]
created: 2026-01-15
last_validated: 2026-03-01
confidence: medium
origins:
  - project: recommendation-engine
    date: 2026-01-15
    context: "Refactoring monolithic training script into composable pipeline stages"
supersedes: []
---

Split the monolithic training script into discrete stages: data loading, feature engineering, model training, evaluation. Each stage reads from and writes to a versioned artifact store (e.g., a timestamped directory in S3).

Use a configuration object (dataclass or Pydantic model) rather than CLI arguments for pipeline parameters. This makes the configuration serializable, diffable, and reproducible — critical for experiment tracking.

The original pipeline failed silently when feature columns were missing. Adding schema validation (pandera or great_expectations) at stage boundaries catches data issues before they propagate to training.
```

`eval/corpus/entries/projects-react-dashboard.md`:
```markdown
---
title: React Dashboard Architecture Ideas
tags: [react, typescript, dashboard, frontend]
created: 2026-03-20
last_validated: 2026-03-20
confidence: prospective
source: horizon-scan
supersedes: []
---

For the upcoming metrics dashboard, consider using React Server Components for the data-heavy panels. RSC would eliminate the client-side fetch waterfall for dashboard widgets that load independent data sources.

Tanstack Table (formerly React Table) handles virtualized rendering for large datasets. If the dashboard needs to display 10k+ rows, this avoids the DOM explosion that crashes standard table components.
```

`eval/corpus/entries/projects-microservices-auth.md`:
```markdown
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
```

- [ ] **Step 6: Verify all entries parse correctly**

Run a quick Rust check that every entry in `eval/corpus/entries/` can be parsed by `Entry::parse()`:

```bash
cd eval && cargo run -- --corpus corpus 2>&1 || echo "will verify after corpus loader is implemented"
```

For now, manually verify a few entries parse by running the test from the parent crate:

```bash
cd /path/to/Mnemosyne && cargo test --test entry_test
```

Expected: All existing entry tests still pass (we haven't modified any source code).

- [ ] **Step 7: Commit**

```bash
git add eval/corpus/entries/
git commit -m "feat(eval): add 39 benchmark corpus entries across 5 axes"
```

---

### Task 3: Create benchmark fixtures

**Files:**
- Create: `eval/corpus/queries.yaml`
- Create: `eval/corpus/contradictions.yaml`
- Create: `eval/corpus/projects/rust-web-api/` (Cargo.toml, src/main.rs, expected.yaml)
- Create: `eval/corpus/projects/python-ml-pipeline/` (pyproject.toml, src/__init__.py, expected.yaml)
- Create: `eval/corpus/projects/haskell-web-app/` (stack.yaml, haskell-web-app.cabal, expected.yaml)
- Create: `eval/corpus/projects/rust-python-mixed/` (Cargo.toml, pyproject.toml, expected.yaml)

- [ ] **Step 1: Create queries.yaml**

20 queries with graded relevance. Entry filenames must match exactly. Context-based queries: q06, q10, q18. Hard queries (low tag overlap, high semantic relevance): q14 (haskell-monads for type systems), q15 (event-sourcing for migration).

`eval/corpus/queries.yaml`:
```yaml
queries:
  - id: q01
    text: "async error handling"
    tags: [async, error-handling]
    context: null
    relevant:
      - entry: techniques-async-error-handling.md
        relevance: 2
      - entry: languages-rust-error-handling.md
        relevance: 1
      - entry: languages-python-async.md
        relevance: 1

  - id: q02
    text: "database performance"
    tags: [database, performance]
    context: null
    relevant:
      - entry: tools-postgres-indexing.md
        relevance: 2
      - entry: techniques-connection-pooling.md
        relevance: 2
      - entry: domains-database-connections.md
        relevance: 1

  - id: q03
    text: "rust async patterns"
    tags: [rust, async]
    context: null
    relevant:
      - entry: languages-rust-async.md
        relevance: 2
      - entry: languages-rust-async-updated.md
        relevance: 2
      - entry: projects-apianyware-async.md
        relevance: 1
      - entry: techniques-async-error-handling.md
        relevance: 1

  - id: q04
    text: "caching best practices"
    tags: [caching]
    context: null
    relevant:
      - entry: techniques-caching-strategies.md
        relevance: 2
      - entry: domains-web-caching.md
        relevance: 2
      - entry: domains-web-caching-aggressive.md
        relevance: 1
      - entry: tools-ci-caching.md
        relevance: 1

  - id: q05
    text: "authentication and security"
    tags: [authentication, security]
    context: null
    relevant:
      - entry: domains-security-auth.md
        relevance: 2
      - entry: projects-microservices-auth.md
        relevance: 1

  - id: q06
    text: "python data processing"
    tags: [python, data]
    context: "python-ml-pipeline"
    relevant:
      - entry: domains-ml-data-pipelines.md
        relevance: 2
      - entry: projects-ml-pipeline-v2.md
        relevance: 1
      - entry: languages-python-typing.md
        relevance: 1

  - id: q07
    text: "container optimization"
    tags: [docker, optimization]
    context: null
    relevant:
      - entry: tools-docker-layers.md
        relevance: 2

  - id: q08
    text: "testing strategies"
    tags: [testing]
    context: null
    relevant:
      - entry: techniques-testing-integration.md
        relevance: 2
      - entry: techniques-testing-mocks.md
        relevance: 1
      - entry: techniques-dependency-injection.md
        relevance: 1

  - id: q09
    text: "distributed system resilience"
    tags: [distributed, resilience]
    context: null
    relevant:
      - entry: techniques-retry-backoff.md
        relevance: 2
      - entry: domains-distributed-systems.md
        relevance: 1

  - id: q10
    text: "state management in frontend"
    tags: [frontend, state-management]
    context: "typescript-react"
    relevant:
      - entry: domains-frontend-state.md
        relevance: 2
      - entry: projects-react-dashboard.md
        relevance: 1

  - id: q11
    text: "memory safety in Rust"
    tags: [rust, memory]
    context: null
    relevant:
      - entry: languages-rust-lifetimes.md
        relevance: 2

  - id: q12
    text: "API rate limiting"
    tags: [api, rate-limiting]
    context: null
    relevant:
      - entry: techniques-rate-limiting.md
        relevance: 2
      - entry: domains-api-design.md
        relevance: 1

  - id: q13
    text: "infrastructure as code"
    tags: [terraform, infrastructure]
    context: null
    relevant:
      - entry: tools-terraform-modules.md
        relevance: 2

  - id: q14
    text: "type systems and generics"
    tags: [type-system, generics]
    context: null
    relevant:
      - entry: languages-typescript-generics.md
        relevance: 2
      - entry: languages-haskell-monads.md
        relevance: 1
      - entry: languages-python-typing.md
        relevance: 1

  - id: q15
    text: "project migration challenges"
    tags: [migration, legacy]
    context: null
    relevant:
      - entry: projects-legacy-migration.md
        relevance: 2
      - entry: techniques-event-sourcing.md
        relevance: 1

  - id: q16
    text: "git branching strategy"
    tags: [git, branching]
    context: null
    relevant:
      - entry: tools-git-workflows.md
        relevance: 2

  - id: q17
    text: "Rust project structure"
    tags: [cargo, rust]
    context: null
    relevant:
      - entry: tools-cargo-workspaces.md
        relevance: 2

  - id: q18
    text: "system observability"
    tags: [observability, monitoring]
    context: "rust-web-api"
    relevant:
      - entry: domains-observability.md
        relevance: 2

  - id: q19
    text: "performance optimization techniques"
    tags: [performance, optimization]
    context: null
    relevant:
      - entry: techniques-zero-copy.md
        relevance: 2
      - entry: tools-postgres-indexing.md
        relevance: 1
      - entry: techniques-caching-strategies.md
        relevance: 1

  - id: q20
    text: "debugging tools and techniques"
    tags: [debugging]
    context: null
    relevant:
      - entry: tools-vscode-debugging.md
        relevance: 2
```

- [ ] **Step 2: Create contradictions.yaml**

`eval/corpus/contradictions.yaml`:
```yaml
pairs:
  - entry_a: languages-rust-async.md
    entry_b: languages-rust-async-updated.md
    is_contradiction: true
    note: "Updated entry supersedes original on cancellation semantics — recommends CancellationToken over broadcast channels"

  - entry_a: domains-web-caching.md
    entry_b: domains-web-caching-aggressive.md
    is_contradiction: true
    note: "Contradictory caching TTL advice — conservative short TTLs vs aggressive immutable caching"

  - entry_a: techniques-testing-integration.md
    entry_b: techniques-testing-mocks.md
    is_contradiction: true
    note: "Opposing stances on whether to mock or integrate real databases for testing"

  - entry_a: techniques-connection-pooling.md
    entry_b: domains-database-connections.md
    is_contradiction: false
    note: "Complementary: pool sizing strategy vs connection lifecycle management — different aspects of the same system"

  - entry_a: languages-rust-error-handling.md
    entry_b: techniques-async-error-handling.md
    is_contradiction: false
    note: "Different scope — Rust-specific error type patterns vs async pipeline error propagation"

  - entry_a: domains-security-auth.md
    entry_b: projects-microservices-auth.md
    is_contradiction: false
    note: "General JWT security practices vs project-specific microservices auth architecture — complementary"

  - entry_a: languages-python-async.md
    entry_b: languages-python-typing.md
    is_contradiction: false
    note: "Near-miss: both Python, but asyncio pitfalls vs type annotation strategies — completely different topics"

  - entry_a: techniques-caching-strategies.md
    entry_b: tools-ci-caching.md
    is_contradiction: false
    note: "Near-miss: share caching tag but application-level caching vs CI build caching — different domains"
```

- [ ] **Step 3: Create mock project — rust-web-api**

`eval/corpus/projects/rust-web-api/Cargo.toml`:
```toml
[package]
name = "rust-web-api"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

`eval/corpus/projects/rust-web-api/src/main.rs`:
```rust
fn main() {}
```

`eval/corpus/projects/rust-web-api/expected.yaml`:
```yaml
languages: [rust]
dependencies: [axum, serde, tokio]
expected_tags: [api, async, concurrency, http, rust, serde, serialization, tokio, web]
```

The expected_tags are derived from `Config::default().context_mappings["cargo_dependencies"]`:
- tokio → [async, tokio, concurrency]
- serde → [serialization, serde]
- axum → [web, http, api]
- Plus the language signal: rust

- [ ] **Step 4: Create mock project — python-ml-pipeline**

`eval/corpus/projects/python-ml-pipeline/pyproject.toml`:
```toml
[project]
name = "ml-pipeline"
version = "0.1.0"

dependencies = [
    "numpy>=1.24",
    "pandas>=2.0",
    "scikit-learn>=1.3",
]
```

`eval/corpus/projects/python-ml-pipeline/src/__init__.py`:
```python
```

`eval/corpus/projects/python-ml-pipeline/expected.yaml`:
```yaml
languages: [python]
dependencies: [numpy, pandas, scikit-learn]
expected_tags: [numpy, pandas, python, scikit-learn]
```

Note: No pyproject_dependencies in `Config::default().context_mappings`, so dependencies map directly to tags via `SignalMapper`'s fallback branch.

- [ ] **Step 5: Create mock project — haskell-web-app**

`eval/corpus/projects/haskell-web-app/stack.yaml`:
```yaml
resolver: lts-21.25
packages:
  - .
```

`eval/corpus/projects/haskell-web-app/haskell-web-app.cabal`:
```
cabal-version: 3.0
name: haskell-web-app
version: 0.1.0.0
build-type: Simple

executable haskell-web-app
  main-is: Main.hs
  build-depends: base >=4.14
```

`eval/corpus/projects/haskell-web-app/expected.yaml`:
```yaml
languages: [haskell]
dependencies: []
expected_tags: [haskell]
```

Note: Haskell's `dependency_parser` is "cabal" but `detect.rs` only handles "cargo" and "pyproject" parsers, so no dependencies are extracted.

- [ ] **Step 6: Create mock project — rust-python-mixed**

`eval/corpus/projects/rust-python-mixed/Cargo.toml`:
```toml
[package]
name = "rust-python-mixed"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
```

`eval/corpus/projects/rust-python-mixed/pyproject.toml`:
```toml
[project]
name = "python-bindings"
version = "0.1.0"

dependencies = [
    "pyo3>=0.20",
]
```

`eval/corpus/projects/rust-python-mixed/expected.yaml`:
```yaml
languages: [python, rust]
dependencies: [pyo3, serde]
expected_tags: [pyo3, python, rust, serde, serialization]
```

- [ ] **Step 7: Commit**

```bash
git add eval/corpus/queries.yaml eval/corpus/contradictions.yaml eval/corpus/projects/
git commit -m "feat(eval): add queries, contradiction pairs, and mock projects to benchmark corpus"
```

---

### Task 4: Implement corpus loader

**Files:**
- Modify: `eval/src/corpus.rs`
- Modify: `eval/src/main.rs` (add module declaration, already present)

- [ ] **Step 1: Write tests for corpus loading**

`eval/src/corpus.rs`:
```rust
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use mnemosyne::knowledge::entry::Entry;

#[derive(Debug, Deserialize)]
pub struct QuerySet {
    pub queries: Vec<QuerySpec>,
}

#[derive(Debug, Deserialize)]
pub struct QuerySpec {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub context: Option<String>,
    #[serde(default)]
    pub relevant: Vec<RelevanceJudgement>,
}

#[derive(Debug, Deserialize)]
pub struct RelevanceJudgement {
    pub entry: String,
    pub relevance: u8,
}

#[derive(Debug, Deserialize)]
pub struct ContradictionSet {
    pub pairs: Vec<ContradictionPair>,
}

#[derive(Debug, Deserialize)]
pub struct ContradictionPair {
    pub entry_a: String,
    pub entry_b: String,
    pub is_contradiction: bool,
    pub note: String,
}

#[derive(Debug, Deserialize)]
pub struct ExpectedContext {
    pub languages: Vec<String>,
    pub dependencies: Vec<String>,
    pub expected_tags: Vec<String>,
}

pub struct ProjectFixture {
    pub path: PathBuf,
    pub expected: ExpectedContext,
}

pub struct Corpus {
    pub entries: Vec<Entry>,
    pub entry_map: HashMap<String, usize>,
    pub queries: QuerySet,
    pub contradictions: ContradictionSet,
    pub projects: Vec<ProjectFixture>,
}

impl Corpus {
    pub fn load(corpus_dir: &Path) -> Result<Self> {
        let entries_dir = corpus_dir.join("entries");
        let entries = Self::load_entries(&entries_dir)?;
        let entry_map = entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                e.file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|name| (name.to_string_lossy().to_string(), i))
            })
            .collect();

        let queries_path = corpus_dir.join("queries.yaml");
        let queries: QuerySet = serde_yaml::from_str(
            &fs::read_to_string(&queries_path)
                .with_context(|| format!("reading {}", queries_path.display()))?,
        )
        .context("parsing queries.yaml")?;

        let contradictions_path = corpus_dir.join("contradictions.yaml");
        let contradictions: ContradictionSet = serde_yaml::from_str(
            &fs::read_to_string(&contradictions_path)
                .with_context(|| format!("reading {}", contradictions_path.display()))?,
        )
        .context("parsing contradictions.yaml")?;

        let projects = Self::load_projects(&corpus_dir.join("projects"))?;

        Ok(Corpus {
            entries,
            entry_map,
            queries,
            contradictions,
            projects,
        })
    }

    fn load_entries(entries_dir: &Path) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        let mut paths: Vec<_> = fs::read_dir(entries_dir)
            .with_context(|| format!("reading {}", entries_dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "md")
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();
        paths.sort();

        for path in paths {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let mut entry = Entry::parse(&content)
                .with_context(|| format!("parsing {}", path.display()))?;
            entry.file_path = Some(path);
            entries.push(entry);
        }
        Ok(entries)
    }

    fn load_projects(projects_dir: &Path) -> Result<Vec<ProjectFixture>> {
        let mut projects = Vec::new();
        if !projects_dir.exists() {
            return Ok(projects);
        }
        let mut dirs: Vec<_> = fs::read_dir(projects_dir)
            .with_context(|| format!("reading {}", projects_dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();
        dirs.sort();

        for dir in dirs {
            let expected_path = dir.join("expected.yaml");
            if expected_path.exists() {
                let content = fs::read_to_string(&expected_path)
                    .with_context(|| format!("reading {}", expected_path.display()))?;
                let expected: ExpectedContext =
                    serde_yaml::from_str(&content).context("parsing expected.yaml")?;
                projects.push(ProjectFixture {
                    path: dir,
                    expected,
                });
            }
        }
        Ok(projects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_corpus_from_test_directory() {
        // This test runs against the real corpus directory
        let corpus_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus");
        if !corpus_dir.exists() {
            // Skip if corpus not yet created
            return;
        }
        let corpus = Corpus::load(&corpus_dir).expect("corpus should load");
        assert!(corpus.entries.len() >= 30, "expected at least 30 entries");
        assert!(
            corpus.queries.queries.len() >= 10,
            "expected at least 10 queries"
        );
        assert!(
            corpus.contradictions.pairs.len() >= 5,
            "expected at least 5 contradiction pairs"
        );
        assert!(
            corpus.projects.len() >= 3,
            "expected at least 3 mock projects"
        );
    }

    #[test]
    fn entry_map_indexes_by_filename() {
        let corpus_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus");
        if !corpus_dir.exists() {
            return;
        }
        let corpus = Corpus::load(&corpus_dir).expect("corpus should load");
        // Spot check: a known entry filename should be in the map
        assert!(
            corpus.entry_map.contains_key("languages-rust-lifetimes.md"),
            "entry_map should contain languages-rust-lifetimes.md"
        );
    }
}
```

- [ ] **Step 2: Verify it compiles and tests pass**

Run: `cd eval && cargo test -- corpus`
Expected: Tests pass (or skip if corpus not yet created).

- [ ] **Step 3: Commit**

```bash
git add eval/src/corpus.rs
git commit -m "feat(eval): implement corpus loader with entry, query, contradiction, and project parsing"
```

---

### Task 5: Implement retrieval metrics

**Files:**
- Modify: `eval/src/retrieval.rs`

The retrieval module computes MRR, Precision@k, Recall@k, and nDCG@k. Low-level metric functions are tested with hardcoded inputs for correctness. The high-level `evaluate_retrieval` function orchestrates corpus loading, search execution, and metric aggregation.

- [ ] **Step 1: Write tests for metric functions**

- [ ] **Step 2: Implement retrieval metrics**

`eval/src/retrieval.rs`:
```rust
use std::collections::{HashMap, HashSet};

use mnemosyne::knowledge::entry::Tag;
use mnemosyne::knowledge::index::{FileIndex, KnowledgeIndex, Query};

use crate::corpus::Corpus;

#[derive(Debug, Clone)]
pub struct PerQueryMetrics {
    pub query_id: String,
    pub reciprocal_rank: f64,
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub ndcg_at_k: f64,
}

#[derive(Debug)]
pub struct RetrievalMetrics {
    pub mrr: f64,
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub ndcg_at_k: f64,
    pub k: usize,
    pub query_count: usize,
    pub per_query: Vec<PerQueryMetrics>,
}

/// Reciprocal rank: 1/position of the first relevant result, or 0 if none.
fn reciprocal_rank(ranked: &[String], relevant: &HashSet<String>) -> f64 {
    for (i, entry) in ranked.iter().enumerate() {
        if relevant.contains(entry) {
            return 1.0 / (i as f64 + 1.0);
        }
    }
    0.0
}

/// Fraction of top-k results that are relevant.
fn precision_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    let top_k = &ranked[..ranked.len().min(k)];
    if top_k.is_empty() {
        return 0.0;
    }
    let hits = top_k.iter().filter(|e| relevant.contains(e.as_str())).count();
    hits as f64 / k as f64
}

/// Fraction of all relevant entries appearing in top-k.
fn recall_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let top_k = &ranked[..ranked.len().min(k)];
    let hits = top_k.iter().filter(|e| relevant.contains(e.as_str())).count();
    hits as f64 / relevant.len() as f64
}

/// Normalised Discounted Cumulative Gain at k.
/// `ranked_relevance`: relevance grade (0/1/2) for each result in ranked order.
fn ndcg_at_k(ranked_relevance: &[u8], ideal_relevance: &[u8], k: usize) -> f64 {
    let dcg = dcg(ranked_relevance, k);
    let idcg = dcg(ideal_relevance, k);
    if idcg == 0.0 {
        return 0.0;
    }
    dcg / idcg
}

fn dcg(relevance: &[u8], k: usize) -> f64 {
    relevance
        .iter()
        .take(k)
        .enumerate()
        .map(|(i, &rel)| {
            let gain = (2.0_f64).powi(rel as i32) - 1.0;
            let discount = (i as f64 + 2.0).log2();
            gain / discount
        })
        .sum()
}

pub fn evaluate_retrieval(corpus: &Corpus, k: usize) -> RetrievalMetrics {
    let index = FileIndex::from_entries(corpus.entries.clone());
    let mut per_query = Vec::new();

    for query_spec in &corpus.queries.queries {
        let query = Query {
            text: Some(query_spec.text.clone()),
            tags: query_spec.tags.iter().map(|t| t.clone() as Tag).collect(),
        };

        let results = index.search(&query);
        let ranked: Vec<String> = results
            .iter()
            .filter_map(|r| {
                r.entry
                    .file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
            })
            .collect();

        let relevant_set: HashSet<String> = query_spec
            .relevant
            .iter()
            .map(|r| r.entry.clone())
            .collect();

        // Build relevance map for nDCG
        let relevance_map: HashMap<String, u8> = query_spec
            .relevant
            .iter()
            .map(|r| (r.entry.clone(), r.relevance))
            .collect();

        let ranked_rel: Vec<u8> = ranked
            .iter()
            .map(|e| *relevance_map.get(e).unwrap_or(&0))
            .collect();

        // Ideal: sort all relevance grades descending
        let mut ideal_rel: Vec<u8> = query_spec.relevant.iter().map(|r| r.relevance).collect();
        ideal_rel.sort_by(|a, b| b.cmp(a));

        let rr = reciprocal_rank(&ranked, &relevant_set);
        let p_at_k = precision_at_k(&ranked, &relevant_set, k);
        let r_at_k = recall_at_k(&ranked, &relevant_set, k);
        let ndcg = ndcg_at_k(&ranked_rel, &ideal_rel, k);

        per_query.push(PerQueryMetrics {
            query_id: query_spec.id.clone(),
            reciprocal_rank: rr,
            precision_at_k: p_at_k,
            recall_at_k: r_at_k,
            ndcg_at_k: ndcg,
        });
    }

    let n = per_query.len() as f64;
    RetrievalMetrics {
        mrr: per_query.iter().map(|q| q.reciprocal_rank).sum::<f64>() / n,
        precision_at_k: per_query.iter().map(|q| q.precision_at_k).sum::<f64>() / n,
        recall_at_k: per_query.iter().map(|q| q.recall_at_k).sum::<f64>() / n,
        ndcg_at_k: per_query.iter().map(|q| q.ndcg_at_k).sum::<f64>() / n,
        k,
        query_count: per_query.len(),
        per_query,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reciprocal_rank_first() {
        let ranked = vec!["a".into(), "b".into(), "c".into()];
        let relevant: HashSet<String> = ["a".into()].into();
        assert_eq!(reciprocal_rank(&ranked, &relevant), 1.0);
    }

    #[test]
    fn test_reciprocal_rank_second() {
        let ranked = vec!["b".into(), "a".into(), "c".into()];
        let relevant: HashSet<String> = ["a".into()].into();
        assert_eq!(reciprocal_rank(&ranked, &relevant), 0.5);
    }

    #[test]
    fn test_reciprocal_rank_not_found() {
        let ranked = vec!["b".into(), "c".into()];
        let relevant: HashSet<String> = ["a".into()].into();
        assert_eq!(reciprocal_rank(&ranked, &relevant), 0.0);
    }

    #[test]
    fn test_precision_at_k() {
        let ranked = vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
        let relevant: HashSet<String> = ["a".into(), "c".into(), "e".into()].into();
        assert!((precision_at_k(&ranked, &relevant, 5) - 0.6).abs() < 1e-10);
        assert!((precision_at_k(&ranked, &relevant, 2) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_recall_at_k() {
        let ranked = vec!["a".into(), "b".into(), "c".into()];
        let relevant: HashSet<String> = ["a".into(), "c".into(), "f".into()].into();
        assert!((recall_at_k(&ranked, &relevant, 3) - 2.0 / 3.0).abs() < 1e-10);
        assert!((recall_at_k(&ranked, &relevant, 1) - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_ndcg_perfect_ranking() {
        let ranked = vec![2, 1, 0];
        let ideal = vec![2, 1, 0];
        assert!((ndcg_at_k(&ranked, &ideal, 3) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ndcg_imperfect_ranking() {
        let ranked = vec![0, 2, 1];
        let ideal = vec![2, 1, 0];
        let ndcg = ndcg_at_k(&ranked, &ideal, 3);
        assert!(ndcg < 1.0, "imperfect ranking should have nDCG < 1.0");
        assert!(ndcg > 0.0, "non-empty results should have nDCG > 0.0");
    }

    #[test]
    fn test_dcg_known_values() {
        // DCG for [2, 0, 1] at k=3:
        // (2^2 - 1)/log2(2) + (2^0 - 1)/log2(3) + (2^1 - 1)/log2(4)
        // = 3/1 + 0/1.585 + 1/2
        // = 3.0 + 0.0 + 0.5 = 3.5
        let relevance = vec![2, 0, 1];
        let result = dcg(&relevance, 3);
        assert!((result - 3.5).abs() < 1e-10);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cd eval && cargo test -- retrieval`
Expected: All 7 tests pass.

- [ ] **Step 4: Commit**

```bash
git add eval/src/retrieval.rs
git commit -m "feat(eval): implement retrieval metrics — MRR, precision@k, recall@k, nDCG@k"
```

---

### Task 6: Implement contradiction metrics

**Files:**
- Modify: `eval/src/contradiction.rs`

Uses `mnemosyne::evolution::contradiction::ContradictionDetector` directly (not `FileIndex::find_contradictions`) to enable threshold parameterization.

- [ ] **Step 1: Write tests and implementation**

`eval/src/contradiction.rs`:
```rust
use mnemosyne::evolution::contradiction::ContradictionDetector;
use mnemosyne::knowledge::entry::Entry;

use crate::corpus::Corpus;

#[derive(Debug, Clone)]
pub struct ContradictionMetrics {
    pub threshold: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub pair_count: usize,
}

pub fn evaluate_contradictions(corpus: &Corpus, threshold: f64) -> ContradictionMetrics {
    let detector = ContradictionDetector::new(threshold);
    let mut true_positives = 0;
    let mut false_positives = 0;
    let mut false_negatives = 0;

    for pair in &corpus.contradictions.pairs {
        let entry_a = corpus.entry_map.get(&pair.entry_a).map(|&i| &corpus.entries[i]);
        let entry_b = corpus.entry_map.get(&pair.entry_b).map(|&i| &corpus.entries[i]);

        let (Some(entry_a), Some(entry_b)) = (entry_a, entry_b) else {
            continue;
        };

        let results = detector.detect(std::slice::from_ref(entry_a), entry_b);
        let flagged = !results.is_empty();

        match (flagged, pair.is_contradiction) {
            (true, true) => true_positives += 1,
            (true, false) => false_positives += 1,
            (false, true) => false_negatives += 1,
            (false, false) => {} // true negative
        }
    }

    let precision = if true_positives + false_positives > 0 {
        true_positives as f64 / (true_positives + false_positives) as f64
    } else {
        0.0
    };
    let recall = if true_positives + false_negatives > 0 {
        true_positives as f64 / (true_positives + false_negatives) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    ContradictionMetrics {
        threshold,
        precision,
        recall,
        f1,
        pair_count: corpus.contradictions.pairs.len(),
    }
}

pub fn sweep_thresholds(corpus: &Corpus) -> Vec<ContradictionMetrics> {
    let mut results = Vec::new();
    let mut threshold = 0.30;
    while threshold <= 0.80 + 1e-9 {
        results.push(evaluate_contradictions(corpus, threshold));
        threshold += 0.05;
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mnemosyne::knowledge::entry::{Confidence, Entry};
    use crate::corpus::{ContradictionPair, ContradictionSet, Corpus, QuerySet};
    use std::collections::HashMap;

    fn make_entry(title: &str, tags: Vec<&str>, filename: &str) -> Entry {
        Entry {
            title: title.to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            created: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            last_validated: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            confidence: Confidence::High,
            source: None,
            origins: vec![],
            supersedes: vec![],
            body: String::new(),
            file_path: Some(std::path::PathBuf::from(filename)),
        }
    }

    fn make_test_corpus() -> Corpus {
        let entry_a = make_entry("A", vec!["rust", "async", "tokio"], "a.md");
        let entry_b = make_entry("B", vec!["rust", "async", "tokio", "cancel"], "b.md");
        let entry_c = make_entry("C", vec!["python", "typing"], "c.md");
        let entries = vec![entry_a, entry_b, entry_c];
        let entry_map: HashMap<String, usize> = [
            ("a.md".into(), 0),
            ("b.md".into(), 1),
            ("c.md".into(), 2),
        ]
        .into();

        Corpus {
            entries,
            entry_map,
            queries: QuerySet { queries: vec![] },
            contradictions: ContradictionSet {
                pairs: vec![
                    ContradictionPair {
                        entry_a: "a.md".into(),
                        entry_b: "b.md".into(),
                        is_contradiction: true,
                        note: "true contradiction".into(),
                    },
                    ContradictionPair {
                        entry_a: "a.md".into(),
                        entry_b: "c.md".into(),
                        is_contradiction: false,
                        note: "no overlap".into(),
                    },
                ],
            },
            projects: vec![],
        }
    }

    #[test]
    fn test_evaluate_known_pairs() {
        let corpus = make_test_corpus();
        // a.md [rust, async, tokio] vs b.md [rust, async, tokio, cancel]
        // Jaccard = 3/4 = 0.75 → flagged at threshold 0.5
        // a.md vs c.md [python, typing] → Jaccard = 0 → not flagged
        let metrics = evaluate_contradictions(&corpus, 0.5);
        assert_eq!(metrics.precision, 1.0); // 1 TP, 0 FP
        assert_eq!(metrics.recall, 1.0); // 1 TP, 0 FN
        assert_eq!(metrics.f1, 1.0);
    }

    #[test]
    fn test_high_threshold_misses_contradiction() {
        let corpus = make_test_corpus();
        // At threshold 0.8, Jaccard 0.75 is below → not flagged
        let metrics = evaluate_contradictions(&corpus, 0.8);
        assert_eq!(metrics.recall, 0.0);
    }

    #[test]
    fn test_sweep_returns_multiple_thresholds() {
        let corpus = make_test_corpus();
        let sweep = sweep_thresholds(&corpus);
        assert!(sweep.len() >= 10);
        assert!((sweep[0].threshold - 0.30).abs() < 1e-9);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd eval && cargo test -- contradiction`
Expected: All 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add eval/src/contradiction.rs
git commit -m "feat(eval): implement contradiction metrics with F1 and threshold sweep"
```

---

### Task 7: Implement context detection metrics

**Files:**
- Modify: `eval/src/context.rs`

- [ ] **Step 1: Write tests and implementation**

`eval/src/context.rs`:
```rust
use std::collections::HashSet;

use mnemosyne::config::Config;
use mnemosyne::context::detect::ProjectDetector;
use mnemosyne::context::detect::Signal;
use mnemosyne::context::mapping::SignalMapper;

use crate::corpus::Corpus;

#[derive(Debug)]
pub struct ContextMetrics {
    pub language_accuracy: f64,
    pub dependency_accuracy: f64,
    pub tag_mapping_accuracy: f64,
    pub project_count: usize,
}

pub fn evaluate_context(corpus: &Corpus) -> ContextMetrics {
    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let mapper = SignalMapper::new(&config);

    let mut language_correct = 0;
    let mut language_total = 0;
    let mut dep_correct = 0;
    let mut dep_total = 0;
    let mut tag_correct = 0;
    let mut tag_total = 0;

    for project in &corpus.projects {
        let signals = match detector.detect(&project.path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Language accuracy
        let detected_languages: HashSet<String> = signals
            .iter()
            .filter_map(|s| match s {
                Signal::Language(lang) => Some(lang.clone()),
                _ => None,
            })
            .collect();
        let expected_languages: HashSet<String> =
            project.expected.languages.iter().cloned().collect();

        language_total += 1;
        if detected_languages == expected_languages {
            language_correct += 1;
        }

        // Dependency accuracy
        let detected_deps: HashSet<String> = signals
            .iter()
            .filter_map(|s| match s {
                Signal::Dependency { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();
        let expected_deps: HashSet<String> =
            project.expected.dependencies.iter().cloned().collect();

        for dep in &expected_deps {
            dep_total += 1;
            if detected_deps.contains(dep) {
                dep_correct += 1;
            }
        }

        // Tag mapping accuracy
        let mapped_tags: HashSet<String> =
            mapper.map_signals(&signals).into_iter().collect();
        let expected_tags: HashSet<String> =
            project.expected.expected_tags.iter().cloned().collect();

        for tag in &expected_tags {
            tag_total += 1;
            if mapped_tags.contains(tag) {
                tag_correct += 1;
            }
        }
    }

    ContextMetrics {
        language_accuracy: if language_total > 0 {
            language_correct as f64 / language_total as f64
        } else {
            0.0
        },
        dependency_accuracy: if dep_total > 0 {
            dep_correct as f64 / dep_total as f64
        } else {
            0.0
        },
        tag_mapping_accuracy: if tag_total > 0 {
            tag_correct as f64 / tag_total as f64
        } else {
            0.0
        },
        project_count: corpus.projects.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy_computation() {
        // Accuracy is a simple fraction — test the math
        let correct = 3.0;
        let total = 4.0;
        assert!((correct / total - 0.75).abs() < 1e-10);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd eval && cargo test -- context`
Expected: Pass.

- [ ] **Step 3: Commit**

```bash
git add eval/src/context.rs
git commit -m "feat(eval): implement context detection metrics — language, dependency, tag accuracy"
```

---

### Task 8: Implement report module and wire up CLI

**Files:**
- Modify: `eval/src/report.rs`
- Modify: `eval/src/main.rs`

- [ ] **Step 1: Implement report formatting**

`eval/src/report.rs`:
```rust
use serde::Serialize;

use crate::contradiction::ContradictionMetrics;
use crate::context::ContextMetrics;
use crate::retrieval::RetrievalMetrics;

#[derive(Serialize)]
struct JsonReport {
    retrieval: JsonRetrieval,
    contradiction: JsonContradiction,
    context_detection: JsonContext,
}

#[derive(Serialize)]
struct JsonRetrieval {
    mrr: f64,
    precision_at_k: f64,
    recall_at_k: f64,
    ndcg_at_k: f64,
    k: usize,
    query_count: usize,
}

#[derive(Serialize)]
struct JsonContradiction {
    threshold: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    pair_count: usize,
    sweep: Option<Vec<JsonSweepEntry>>,
}

#[derive(Serialize)]
struct JsonSweepEntry {
    threshold: f64,
    precision: f64,
    recall: f64,
    f1: f64,
}

#[derive(Serialize)]
struct JsonContext {
    language_accuracy: f64,
    dependency_accuracy: f64,
    tag_mapping_accuracy: f64,
    project_count: usize,
}

pub fn format_human(
    retrieval: &RetrievalMetrics,
    contradiction: &ContradictionMetrics,
    context: &ContextMetrics,
    sweep: &Option<Vec<ContradictionMetrics>>,
    verbose: bool,
) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Retrieval Metrics (k={}, {} queries):\n",
        retrieval.k, retrieval.query_count
    ));
    out.push_str(&format!("  MRR:           {:.3}\n", retrieval.mrr));
    out.push_str(&format!("  Precision@{}:   {:.3}\n", retrieval.k, retrieval.precision_at_k));
    out.push_str(&format!("  Recall@{}:      {:.3}\n", retrieval.k, retrieval.recall_at_k));
    out.push_str(&format!("  nDCG@{}:        {:.3}\n", retrieval.k, retrieval.ndcg_at_k));

    if verbose {
        out.push_str("\n  Per-query breakdown:\n");
        for q in &retrieval.per_query {
            out.push_str(&format!(
                "    {}: MRR={:.3}  P@k={:.3}  R@k={:.3}  nDCG={:.3}\n",
                q.query_id, q.reciprocal_rank, q.precision_at_k, q.recall_at_k, q.ndcg_at_k
            ));
        }
    }

    out.push_str(&format!(
        "\nContradiction Detection (threshold={:.2}, {} pairs):\n",
        contradiction.threshold, contradiction.pair_count
    ));
    out.push_str(&format!("  Precision:     {:.3}\n", contradiction.precision));
    out.push_str(&format!("  Recall:        {:.3}\n", contradiction.recall));
    out.push_str(&format!("  F1:            {:.3}\n", contradiction.f1));

    if let Some(sweep) = sweep {
        out.push_str("\n  Threshold Sweep:\n");
        for s in sweep {
            let marker = if (s.threshold - contradiction.threshold).abs() < 0.01 {
                " <-- current default"
            } else {
                ""
            };
            out.push_str(&format!(
                "    {:.2}  P={:.2}  R={:.2}  F1={:.2}{}\n",
                s.threshold, s.precision, s.recall, s.f1, marker
            ));
        }
    }

    out.push_str(&format!(
        "\nContext Detection ({} projects):\n",
        context.project_count
    ));
    out.push_str(&format!("  Language accuracy:    {:.3}\n", context.language_accuracy));
    out.push_str(&format!("  Dependency accuracy:  {:.3}\n", context.dependency_accuracy));
    out.push_str(&format!("  Tag mapping accuracy: {:.3}\n", context.tag_mapping_accuracy));

    out
}

pub fn format_json(
    retrieval: &RetrievalMetrics,
    contradiction: &ContradictionMetrics,
    context: &ContextMetrics,
    sweep: &Option<Vec<ContradictionMetrics>>,
) -> String {
    let report = JsonReport {
        retrieval: JsonRetrieval {
            mrr: retrieval.mrr,
            precision_at_k: retrieval.precision_at_k,
            recall_at_k: retrieval.recall_at_k,
            ndcg_at_k: retrieval.ndcg_at_k,
            k: retrieval.k,
            query_count: retrieval.query_count,
        },
        contradiction: JsonContradiction {
            threshold: contradiction.threshold,
            precision: contradiction.precision,
            recall: contradiction.recall,
            f1: contradiction.f1,
            pair_count: contradiction.pair_count,
            sweep: sweep.as_ref().map(|s| {
                s.iter()
                    .map(|m| JsonSweepEntry {
                        threshold: m.threshold,
                        precision: m.precision,
                        recall: m.recall,
                        f1: m.f1,
                    })
                    .collect()
            }),
        },
        context_detection: JsonContext {
            language_accuracy: context.language_accuracy,
            dependency_accuracy: context.dependency_accuracy,
            tag_mapping_accuracy: context.tag_mapping_accuracy,
            project_count: context.project_count,
        },
    };
    serde_json::to_string_pretty(&report).expect("JSON serialization should not fail")
}
```

- [ ] **Step 2: Wire up main.rs**

`eval/src/main.rs`:
```rust
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod contradiction;
mod context;
mod corpus;
mod report;
mod retrieval;

#[derive(Parser)]
#[command(name = "mnemosyne-eval")]
struct Cli {
    /// Path to corpus directory
    #[arg(long, default_value = "corpus")]
    corpus: PathBuf,

    /// Top-k for retrieval metrics
    #[arg(long, default_value = "5")]
    k: usize,

    /// Run contradiction threshold sweep
    #[arg(long)]
    sweep: bool,

    /// Per-query and per-pair breakdown
    #[arg(long)]
    verbose: bool,

    /// Output in JSON format
    #[arg(long)]
    json: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let corpus = corpus::Corpus::load(&cli.corpus)?;

    let retrieval_metrics = retrieval::evaluate_retrieval(&corpus, cli.k);
    let contradiction_metrics = contradiction::evaluate_contradictions(&corpus, 0.5);
    let context_metrics = context::evaluate_context(&corpus);

    let sweep = if cli.sweep {
        Some(contradiction::sweep_thresholds(&corpus))
    } else {
        None
    };

    if cli.json {
        println!(
            "{}",
            report::format_json(
                &retrieval_metrics,
                &contradiction_metrics,
                &context_metrics,
                &sweep
            )
        );
    } else {
        print!(
            "{}",
            report::format_human(
                &retrieval_metrics,
                &contradiction_metrics,
                &context_metrics,
                &sweep,
                cli.verbose
            )
        );
    }

    Ok(())
}
```

- [ ] **Step 3: Build and run against corpus**

Run: `cd eval && cargo run -- --verbose --sweep`
Expected: Compiles and prints retrieval metrics, contradiction metrics with sweep, and context detection metrics.

- [ ] **Step 4: Run with JSON output**

Run: `cd eval && cargo run -- --json`
Expected: Valid JSON output matching the schema from the spec.

- [ ] **Step 5: Run all tests**

Run: `cd eval && cargo test`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add eval/src/report.rs eval/src/main.rs
git commit -m "feat(eval): implement report formatting and wire up CLI for Rust evaluation harness"
```

---

## Phase 2: Python Quality Harness

### Task 9: Scaffold Python package and create rubric

**Files:**
- Create: `eval/quality/pyproject.toml`
- Create: `eval/quality/rubrics/entry_quality.yaml`
- Create: `eval/quality/src/__init__.py`
- Create: `eval/quality/src/providers/__init__.py`

- [ ] **Step 1: Create pyproject.toml**

`eval/quality/pyproject.toml`:
```toml
[project]
name = "mnemosyne-quality"
version = "0.1.0"
description = "LLM-as-judge quality evaluation for Mnemosyne knowledge entries"
requires-python = ">=3.10"
dependencies = [
    "anthropic>=0.39.0",
    "pyyaml>=6.0",
]

[project.scripts]
mnemosyne-quality = "eval.quality.src.__main__:main"
```

- [ ] **Step 2: Create rubric YAML**

`eval/quality/rubrics/entry_quality.yaml`:
```yaml
name: entry_quality
description: Evaluates the quality of a Mnemosyne knowledge entry
dimensions:
  specificity:
    description: "How specific and targeted is this entry?"
    anchors:
      5: "Identifies a specific technology, version, or scenario. A practitioner could apply this without further research."
      4: "Identifies a specific area but requires some contextual knowledge."
      3: "Reasonably specific but could apply to multiple situations."
      2: "Vague. Could apply to many technologies or scenarios."
      1: "Completely generic. No actionable specificity."

  actionability:
    description: "Could someone act on this knowledge?"
    anchors:
      5: "Contains a concrete recommendation, pattern, or constraint that directly guides implementation."
      4: "Contains useful guidance but requires interpretation."
      3: "Informative but not directly actionable."
      2: "Mostly context or background."
      1: "No actionable content."

  provenance:
    description: "Is the origin and evidence base clear?"
    anchors:
      5: "Multiple dated origins with specific project context. Clear evidence trail."
      4: "At least one origin with context. Provenance is traceable."
      3: "Origin present but context is thin."
      2: "Origin field exists but is minimal or generic."
      1: "No provenance information."

  confidence_fit:
    description: "Does the confidence level match the evidence?"
    anchors:
      5: "Confidence perfectly matches evidence depth (e.g. high confidence with multi-project validation)."
      4: "Confidence is reasonable given the evidence."
      3: "Confidence is slightly optimistic or conservative."
      2: "Confidence is mismatched — e.g. high confidence from a single observation."
      1: "Confidence is clearly wrong given the evidence."
```

- [ ] **Step 3: Create package init files**

`eval/quality/src/__init__.py`:
```python
```

`eval/quality/src/providers/__init__.py`:
```python
```

- [ ] **Step 4: Commit**

```bash
git add eval/quality/
git commit -m "feat(eval): scaffold Python quality harness with rubric YAML"
```

---

### Task 10: Implement structural completeness checks

**Files:**
- Create: `eval/quality/src/structural.py`
- Create: `eval/quality/tests/test_structural.py`

- [ ] **Step 1: Write failing tests**

`eval/quality/tests/__init__.py`:
```python
```

`eval/quality/tests/test_structural.py`:
```python
import pytest
from eval.quality.src.structural import check_entry, StructuralResult


VALID_ENTRY = """\
---
title: Test Entry
tags: [rust, async]
created: 2025-06-01
last_validated: 2025-06-01
confidence: high
origins:
  - project: test-project
    date: 2025-06-01
    context: "Testing"
supersedes: []
---

This is the body content.
"""

MISSING_TITLE = """\
---
tags: [rust]
created: 2025-06-01
last_validated: 2025-06-01
confidence: high
supersedes: []
---

Body.
"""

INVALID_CONFIDENCE = """\
---
title: Bad Confidence
tags: [rust]
created: 2025-06-01
last_validated: 2025-06-01
confidence: extreme
supersedes: []
---

Body.
"""

HIGH_NO_ORIGINS = """\
---
title: High Without Origins
tags: [rust]
created: 2025-06-01
last_validated: 2025-06-01
confidence: high
supersedes: []
---

Body without origins.
"""

PROSPECTIVE_NO_ORIGINS = """\
---
title: Prospective Entry
tags: [rust]
created: 2025-06-01
last_validated: 2025-06-01
confidence: prospective
source: horizon-scan
supersedes: []
---

Body.
"""


def test_valid_entry():
    result = check_entry(VALID_ENTRY, "test.md")
    assert result.valid
    assert result.errors == []


def test_missing_title():
    result = check_entry(MISSING_TITLE, "test.md")
    assert not result.valid
    assert any("title" in e.lower() for e in result.errors)


def test_invalid_confidence():
    result = check_entry(INVALID_CONFIDENCE, "test.md")
    assert not result.valid
    assert any("confidence" in e.lower() for e in result.errors)


def test_high_confidence_without_origins():
    result = check_entry(HIGH_NO_ORIGINS, "test.md")
    assert not result.valid
    assert any("origins" in e.lower() for e in result.errors)


def test_prospective_without_origins_is_ok():
    result = check_entry(PROSPECTIVE_NO_ORIGINS, "test.md")
    assert result.valid
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd eval/quality && python -m pytest tests/test_structural.py -v`
Expected: FAIL (module not found).

- [ ] **Step 3: Implement structural.py**

`eval/quality/src/structural.py`:
```python
from __future__ import annotations

import re
from dataclasses import dataclass, field

import yaml


VALID_CONFIDENCE = {"high", "medium", "low", "prospective"}
ISO_DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


@dataclass
class StructuralResult:
    filename: str
    valid: bool
    errors: list[str] = field(default_factory=list)


def check_entry(content: str, filename: str) -> StructuralResult:
    """Check structural completeness of a knowledge entry."""
    errors: list[str] = []

    # Split frontmatter
    content = content.strip()
    if not content.startswith("---"):
        errors.append("Missing YAML frontmatter delimiter")
        return StructuralResult(filename=filename, valid=False, errors=errors)

    parts = content.split("---", 2)
    if len(parts) < 3:
        errors.append("Missing closing frontmatter delimiter")
        return StructuralResult(filename=filename, valid=False, errors=errors)

    yaml_str = parts[1]
    body = parts[2].strip()

    try:
        fm = yaml.safe_load(yaml_str)
    except yaml.YAMLError as e:
        errors.append(f"Invalid YAML: {e}")
        return StructuralResult(filename=filename, valid=False, errors=errors)

    if not isinstance(fm, dict):
        errors.append("Frontmatter is not a mapping")
        return StructuralResult(filename=filename, valid=False, errors=errors)

    # Required fields
    if "title" not in fm:
        errors.append("Missing required field: title")

    if "tags" not in fm:
        errors.append("Missing required field: tags")
    elif not isinstance(fm["tags"], list) or len(fm["tags"]) == 0:
        errors.append("Tags must be a non-empty list")

    if "created" not in fm:
        errors.append("Missing required field: created")
    elif not ISO_DATE_RE.match(str(fm["created"])):
        errors.append(f"Invalid date format in created: {fm['created']}")

    if "confidence" not in fm:
        errors.append("Missing required field: confidence")
    elif str(fm["confidence"]).lower() not in VALID_CONFIDENCE:
        errors.append(
            f"Invalid confidence value: {fm['confidence']} "
            f"(must be one of {sorted(VALID_CONFIDENCE)})"
        )

    # Body check
    if not body:
        errors.append("Body is empty")

    # Origins check for high/medium confidence
    confidence = str(fm.get("confidence", "")).lower()
    if confidence in ("high", "medium"):
        origins = fm.get("origins", [])
        if not origins:
            errors.append(
                f"Origins should be present for {confidence} confidence entries"
            )

    return StructuralResult(
        filename=filename,
        valid=len(errors) == 0,
        errors=errors,
    )


def check_directory(entries_dir: str) -> list[StructuralResult]:
    """Check all .md files in a directory."""
    import os

    results = []
    for filename in sorted(os.listdir(entries_dir)):
        if not filename.endswith(".md"):
            continue
        filepath = os.path.join(entries_dir, filename)
        with open(filepath) as f:
            content = f.read()
        results.append(check_entry(content, filename))
    return results
```

- [ ] **Step 4: Run tests**

Run: `cd eval/quality && python -m pytest tests/test_structural.py -v`
Expected: All 5 tests pass.

- [ ] **Step 5: Commit**

```bash
git add eval/quality/src/structural.py eval/quality/tests/
git commit -m "feat(eval): implement structural completeness checks for knowledge entries"
```

---

### Task 11: Implement Judge protocol, Claude provider, and rubric loader

**Files:**
- Create: `eval/quality/src/judge.py`
- Create: `eval/quality/src/providers/claude.py`
- Create: `eval/quality/src/rubric.py`

- [ ] **Step 1: Implement Judge protocol**

`eval/quality/src/judge.py`:
```python
from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


@dataclass
class JudgeScore:
    dimension: str
    score: int
    justification: str


class Judge(Protocol):
    def evaluate(self, entry_content: str, rubric_prompt: str) -> list[JudgeScore]:
        """Score a knowledge entry against a rubric prompt."""
        ...
```

- [ ] **Step 2: Implement rubric loader**

`eval/quality/src/rubric.py`:
```python
from __future__ import annotations

import random
from typing import Any

import yaml


def load_rubric(path: str) -> dict[str, Any]:
    """Load a rubric YAML file."""
    with open(path) as f:
        return yaml.safe_load(f)


def format_rubric_prompt(rubric: dict[str, Any], shuffle: bool = False) -> str:
    """Format a rubric into a prompt string for LLM evaluation.

    If shuffle=True, randomize dimension order for variance reduction.
    """
    dimensions = list(rubric.get("dimensions", {}).items())
    if shuffle:
        random.shuffle(dimensions)

    lines = [
        f"Evaluate this knowledge entry on the following dimensions.",
        f"For each dimension, provide a score (1-5) and a one-sentence justification.",
        f"Return your response as a YAML list with fields: dimension, score, justification.",
        "",
    ]

    for dim_name, dim_spec in dimensions:
        lines.append(f"## {dim_name}")
        lines.append(dim_spec.get("description", ""))
        lines.append("")
        for score, anchor in sorted(dim_spec.get("anchors", {}).items(), reverse=True):
            lines.append(f"  {score}: {anchor}")
        lines.append("")

    return "\n".join(lines)
```

- [ ] **Step 3: Implement Claude provider**

`eval/quality/src/providers/claude.py`:
```python
from __future__ import annotations

import yaml

from anthropic import Anthropic

from eval.quality.src.judge import Judge, JudgeScore


class ClaudeJudge:
    """Judge implementation using the Anthropic SDK."""

    def __init__(self, model: str = "claude-haiku-4-5-20251001"):
        self.client = Anthropic()
        self.model = model

    def evaluate(self, entry_content: str, rubric_prompt: str) -> list[JudgeScore]:
        message = self.client.messages.create(
            model=self.model,
            max_tokens=1024,
            messages=[
                {
                    "role": "user",
                    "content": (
                        f"{rubric_prompt}\n\n"
                        f"---\n\n"
                        f"Entry to evaluate:\n\n"
                        f"{entry_content}\n\n"
                        f"---\n\n"
                        f"Respond with ONLY a YAML list. Each item must have: "
                        f"dimension, score (integer 1-5), justification (one sentence)."
                    ),
                }
            ],
        )

        response_text = message.content[0].text
        return self._parse_response(response_text)

    def _parse_response(self, text: str) -> list[JudgeScore]:
        # Strip markdown code fences if present
        text = text.strip()
        if text.startswith("```"):
            lines = text.split("\n")
            text = "\n".join(lines[1:])
            if text.endswith("```"):
                text = text[:-3]

        parsed = yaml.safe_load(text)
        if not isinstance(parsed, list):
            return []

        scores = []
        for item in parsed:
            if isinstance(item, dict) and all(
                k in item for k in ("dimension", "score", "justification")
            ):
                scores.append(
                    JudgeScore(
                        dimension=str(item["dimension"]),
                        score=int(item["score"]),
                        justification=str(item["justification"]),
                    )
                )
        return scores
```

- [ ] **Step 4: Commit**

```bash
git add eval/quality/src/judge.py eval/quality/src/rubric.py eval/quality/src/providers/claude.py
git commit -m "feat(eval): implement Judge protocol, Claude provider, and rubric loader"
```

---

### Task 12: Implement report module, CLI, and config

**Files:**
- Create: `eval/quality/src/report.py`
- Create: `eval/quality/src/config.py`
- Create: `eval/quality/src/__main__.py`

- [ ] **Step 1: Implement config.py**

`eval/quality/src/config.py`:
```python
from __future__ import annotations

import os


def get_provider(cli_provider: str | None) -> str:
    return cli_provider or os.environ.get("MNEMOSYNE_EVAL_PROVIDER", "claude")


def get_model(cli_model: str | None) -> str:
    return cli_model or os.environ.get(
        "MNEMOSYNE_EVAL_MODEL", "claude-haiku-4-5-20251001"
    )
```

- [ ] **Step 2: Implement report.py**

`eval/quality/src/report.py`:
```python
from __future__ import annotations

import json
import statistics
from dataclasses import dataclass, field

from eval.quality.src.judge import JudgeScore
from eval.quality.src.structural import StructuralResult


@dataclass
class EntryReport:
    filename: str
    scores: list[JudgeScore]
    structural: StructuralResult | None = None


@dataclass
class AggregateReport:
    entries: list[EntryReport]
    structural_results: list[StructuralResult] = field(default_factory=list)

    def dimension_stats(self) -> dict[str, dict[str, float]]:
        """Compute mean, median, std for each dimension."""
        by_dim: dict[str, list[int]] = {}
        for entry in self.entries:
            for score in entry.scores:
                by_dim.setdefault(score.dimension, []).append(score.score)

        stats = {}
        for dim, scores in sorted(by_dim.items()):
            stats[dim] = {
                "mean": round(statistics.mean(scores), 1),
                "median": statistics.median(scores),
                "std": round(statistics.stdev(scores), 1) if len(scores) > 1 else 0.0,
            }
        return stats

    def format_human(self, verbose: bool = False) -> str:
        lines = []
        dim_stats = self.dimension_stats()

        lines.append(f"Entry Quality (N={len(self.entries)} entries):")
        for dim, stats in dim_stats.items():
            lines.append(
                f"  {dim + ':':20s} mean={stats['mean']:.1f}  "
                f"median={stats['median']:.0f}  std={stats['std']:.1f}"
            )

        # Structural completeness
        valid = sum(1 for r in self.structural_results if r.valid)
        total = len(self.structural_results)
        issues = [r for r in self.structural_results if not r.valid]
        lines.append(f"\n  Structural completeness: {valid}/{total} valid")
        if issues:
            lines.append(f"  ({len(issues)} issues)")
            for r in issues:
                for err in r.errors:
                    lines.append(f"    - {r.filename}: {err}")

        if verbose:
            # Lowest scoring entries
            entry_avgs = []
            for entry in self.entries:
                if entry.scores:
                    avg = statistics.mean(s.score for s in entry.scores)
                    entry_avgs.append((entry.filename, avg, entry.scores))
            entry_avgs.sort(key=lambda x: x[1])

            lines.append("\n  Lowest scoring entries:")
            for filename, avg, scores in entry_avgs[:5]:
                score_str = ", ".join(
                    f"{s.dimension}={s.score}" for s in scores
                )
                lines.append(f"    - {filename}: {score_str}")

        return "\n".join(lines)

    def format_json(self) -> str:
        return json.dumps(
            {
                "entry_count": len(self.entries),
                "dimension_stats": self.dimension_stats(),
                "structural": {
                    "valid": sum(1 for r in self.structural_results if r.valid),
                    "total": len(self.structural_results),
                },
                "entries": [
                    {
                        "filename": e.filename,
                        "scores": [
                            {
                                "dimension": s.dimension,
                                "score": s.score,
                                "justification": s.justification,
                            }
                            for s in e.scores
                        ],
                    }
                    for e in self.entries
                ],
            },
            indent=2,
        )
```

- [ ] **Step 3: Implement __main__.py**

`eval/quality/src/__main__.py`:
```python
from __future__ import annotations

import argparse
import os
import random
import statistics
import sys

from eval.quality.src.config import get_model, get_provider
from eval.quality.src.judge import JudgeScore
from eval.quality.src.report import AggregateReport, EntryReport
from eval.quality.src.rubric import format_rubric_prompt, load_rubric
from eval.quality.src.structural import check_directory, check_entry


def create_judge(provider: str, model: str):
    if provider == "claude":
        from eval.quality.src.providers.claude import ClaudeJudge

        return ClaudeJudge(model=model)
    else:
        print(f"Unknown provider: {provider}", file=sys.stderr)
        sys.exit(1)


def evaluate_entries(
    entries_dir: str,
    rubric_path: str,
    provider: str,
    model: str,
    single_pass: bool,
    verbose: bool,
) -> AggregateReport:
    rubric = load_rubric(rubric_path)
    judge = create_judge(provider, model)

    # Structural checks first (no API calls)
    structural_results = check_directory(entries_dir)

    entry_reports = []
    for filename in sorted(os.listdir(entries_dir)):
        if not filename.endswith(".md"):
            continue
        filepath = os.path.join(entries_dir, filename)
        with open(filepath) as f:
            content = f.read()

        if verbose:
            print(f"  Evaluating {filename}...", file=sys.stderr)

        # Pass 1: standard dimension order
        prompt1 = format_rubric_prompt(rubric, shuffle=False)
        scores1 = judge.evaluate(content, prompt1)

        if single_pass:
            entry_reports.append(EntryReport(filename=filename, scores=scores1))
            continue

        # Pass 2: shuffled dimension order (variance reduction)
        prompt2 = format_rubric_prompt(rubric, shuffle=True)
        scores2 = judge.evaluate(content, prompt2)

        # Average scores across passes, keep justification from pass 1
        averaged = _average_scores(scores1, scores2)
        entry_reports.append(EntryReport(filename=filename, scores=averaged))

    return AggregateReport(entries=entry_reports, structural_results=structural_results)


def _average_scores(
    pass1: list[JudgeScore], pass2: list[JudgeScore]
) -> list[JudgeScore]:
    """Average scores from two passes, keeping pass 1 justifications."""
    scores2_map = {s.dimension: s.score for s in pass2}
    averaged = []
    for s1 in pass1:
        s2_score = scores2_map.get(s1.dimension, s1.score)
        avg = round(statistics.mean([s1.score, s2_score]))
        averaged.append(
            JudgeScore(
                dimension=s1.dimension,
                score=avg,
                justification=s1.justification,
            )
        )
    return averaged


def main():
    parser = argparse.ArgumentParser(description="Mnemosyne knowledge quality evaluator")
    parser.add_argument(
        "--corpus",
        default=os.path.join(os.path.dirname(__file__), "..", "..", "corpus", "entries"),
        help="Path to entries directory (default: eval/corpus/entries)",
    )
    parser.add_argument("--store", help="Evaluate live knowledge store instead of corpus")
    parser.add_argument(
        "--rubric",
        default=os.path.join(
            os.path.dirname(__file__), "..", "rubrics", "entry_quality.yaml"
        ),
        help="Path to rubric YAML",
    )
    parser.add_argument("--provider", help="LLM provider (default: claude)")
    parser.add_argument("--model", help="Model ID")
    parser.add_argument(
        "--single-pass", action="store_true", help="Skip variance reduction"
    )
    parser.add_argument("--json", action="store_true", help="Output in JSON format")
    parser.add_argument("--verbose", action="store_true", help="Per-entry breakdown")

    args = parser.parse_args()

    entries_dir = args.store if args.store else args.corpus
    provider = get_provider(args.provider)
    model = get_model(args.model)

    report = evaluate_entries(
        entries_dir=entries_dir,
        rubric_path=args.rubric,
        provider=provider,
        model=model,
        single_pass=args.single_pass,
        verbose=args.verbose,
    )

    if args.json:
        print(report.format_json())
    else:
        print(report.format_human(verbose=args.verbose))


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Run structural checks (no API key required)**

Run: `cd eval/quality && python -c "from eval.quality.src.structural import check_directory; results = check_directory('../corpus/entries'); print(f'{sum(r.valid for r in results)}/{len(results)} valid')"`
Expected: `39/39 valid` (all corpus entries pass structural checks).

- [ ] **Step 5: Commit**

```bash
git add eval/quality/src/
git commit -m "feat(eval): implement Python quality harness with LLM-as-judge, variance reduction, and CLI"
```

---

## Phase 3: Multi-Session Simulation (Intent)

**Goal:** Validate that knowledge accumulates correctly and transfers across projects over a simulated multi-session workflow.

**Approach:** A Python-based simulation (extending the Phase 2 package) that:

1. Initialises a fresh Mnemosyne store in a temporary directory
2. Simulates 3-5 sessions across 2-3 mock projects
3. Each session: queries for context, injects synthetic observations via `promote`, triggers `curate` where appropriate
4. Measures knowledge base state at each session boundary using Phase 2 quality metrics and Phase 1 retrieval metrics

**Open design questions:**
- How to simulate the "developer discovers something" step — scripted fixtures or LLM-driven observation generation?
- Whether to test via CLI invocations (black-box) or library calls (white-box)
- How to define expected state at each boundary without making the test tautological

**Success criteria:**
- Cross-project knowledge is retrievable from a project that didn't originate it
- Contradiction detection fires when a later session contradicts an earlier one
- Quality metrics do not degrade across sessions

**Will receive its own detailed spec and plan once Phases 1-2 are proven.**

---

## Phase 4: Controlled Impact Experiments (Intent)

**Goal:** Demonstrate measurably that Mnemosyne improves AI assistant outcomes on coding tasks.

**Approach:** An A/B experimental harness that:

1. Defines 5-10 coding tasks, each containing a known pitfall that cross-project knowledge could prevent
2. Seeds relevant knowledge (treatment) or provides no prior knowledge (control)
3. Runs each task N times per condition via the Claude API
4. Evaluates outcomes: task completion, pitfall avoidance, code quality (LLM-as-judge)
5. Reports effect sizes with confidence intervals

**Open design questions:**
- Task design — what pitfalls are concrete enough to test but general enough to matter?
- Sample size — how many runs per condition for statistical power given LLM variance?
- Cost management — multiple API calls per run; how to keep experiments affordable?
- Whether to use Claude Code as the agent or a simpler agent loop

**Success criteria:**
- Statistically meaningful difference in pitfall avoidance rate between conditions
- Effect demonstrated across at least 3 distinct tasks
- Results reproducible across runs

**Will receive its own detailed spec and plan once Phases 1-2 are proven.**
