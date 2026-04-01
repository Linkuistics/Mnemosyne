# Coding Conventions

## Development Approach
- **TDD** — write tests first, then implement
- **Small files** — each file handles one concern
- **Descriptive names** — long is fine; consistency matters
- **Uniform naming** — don't mix get_thing/fetch_thing; pick one verb

## Code Quality
- Simplicity, readability, reusability, single concern, testability
- No unwrap/expect in production code
- No unbounded channels
- Bounded complexity — don't add features/abstractions beyond what's asked

## Rust-Specific
- `thiserror` for library errors, `anyhow` for CLI/application errors
- `tracing` macros only (not `log` crate)
- `cargo +nightly fmt` before committing
- Import grouping: stdlib → external → local

## Error Handling
- Use `?` operator for propagation
- Provide descriptive error messages with context
- Handle errors gracefully — no panicking unless absolutely necessary

## Async
- Tokio runtime
- Bounded channels only
- No blocking operations in async contexts
