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
