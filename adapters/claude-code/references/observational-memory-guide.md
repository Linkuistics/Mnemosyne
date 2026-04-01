# Observational Memory for LLM-Driven Projects

## The Problem

LLM agents (like Claude) working on multi-session projects lose context between sessions and
fail to accumulate knowledge systematically. Key discoveries get lost because:

1. The LLM forgets to record them (deliberate capture fails)
2. Learnings are trapped in one plan file, invisible to other workstreams
3. No mechanism ensures learnings are consulted when relevant

## The Solution: Observational Memory

Adapted from Mastra's Observational Memory system for AI agents. The core principle:
learning capture must be **structural, not deliberate** — woven into the workflow at
mandatory checkpoints.

### Three Tiers

| Tier | Where | When written | When promoted |
|------|-------|-------------|---------------|
| Raw observations | Plan step Observe field | After every step | During /reflect |
| Durable learnings | knowledge/ axis files | During /reflect | When patterns recur |
| Cross-cutting patterns | Broader knowledge/ axis | During /reflect | Ongoing |

### The Step Cycle: Do → Verify → Observe

Every plan step has three mandatory phases:

- **Do**: The implementation work
- **Verify**: Confirm it works (tests, manual check, integration validation)
- **Observe**: Record what was discovered — not what was *done*, but what was *learned*

Observations use priority codes:
- 🔴 Critical — will break things or waste significant time if not known
- 🟡 Useful — saves time, avoids confusion, clarifies ambiguity
- 🟢 Informational — context, not directly actionable

### Reflection: Code Review as Promotion

Every 3-5 implementation steps, a code review session serves as the Reflector:

1. Review accumulated observations
2. Determine which apply beyond the current task
3. Promote to the appropriate knowledge axis via /reflect
4. Detect patterns across observations
5. Condense related entries

### The Promote-Eagerly Rule

Start observations at the narrowest axis. During reflection, promote upward when a learning
applies more broadly. Always remove from the narrow file when promoting — no duplication.

### CLAUDE.md Routing

Claude Code auto-loads CLAUDE.md files from parent directories. Each CLAUDE.md is a routing
file — small, containing pointers to knowledge files. This is the only mechanism that
guarantees context loading without relying on LLM discipline.

### Knowledge Axes

Every project has its own axes, but common patterns:

- **Primary axis**: the main organisational dimension (services, targets, packages)
- **Secondary axis**: the cross-cutting dimension (features, apps, endpoints)
- **Matrix axis**: the intersection of primary x secondary
- **Infrastructure axes**: pipeline, deployment, testing strategies

## Using the Skills

- `/begin-work <context> [sub-context]` — always start here
- `/reflect` — run during code review sessions
- `/setup-knowledge` — scaffold the system for a new project
- `/create-plan` — create a plan with observational memory built in

## Global Knowledge Integration

When the Mnemosyne CLI is installed, the per-project system gains a global layer:

- `/begin-work` also loads relevant global knowledge from `~/.mnemosyne/`
- `/reflect` offers to promote cross-project learnings to global
- `/promote-global` for ad-hoc promotion
- `/curate-global` for reflective review of global entries
- `/explore-knowledge` for gap analysis and horizon scanning

Global features degrade gracefully — if `mnemosyne` is not installed, all per-project
functionality works exactly as before.
