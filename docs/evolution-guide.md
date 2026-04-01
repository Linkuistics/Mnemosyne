# Knowledge Evolution Guide

This document explains the philosophy and mechanics of how knowledge evolves in Mnemosyne: why it is designed the way it is, how the technical mechanisms work, and how to use them effectively.

## Contents

- [Evidence-Based Evolution vs Time-Based Decay](#evidence-based-evolution-vs-time-based-decay)
- [Why Knowledge Should Not Auto-Expire](#why-knowledge-should-not-auto-expire)
- [Confidence Level Semantics](#confidence-level-semantics)
- [Contradiction Detection](#contradiction-detection)
- [Supersession](#supersession)
- [Divergence Detection](#divergence-detection)
- [Reflective Curation](#reflective-curation)
- [The Socratic Exploration Model](#the-socratic-exploration-model)
- [The Full Evolution Pathway](#the-full-evolution-pathway)

---

## Evidence-Based Evolution vs Time-Based Decay

Most knowledge management systems implement staleness via time: entries older than N days are flagged for review or automatically archived. This is administratively simple but epistemically wrong.

Time alone tells you nothing about whether a piece of knowledge is still accurate. Consider:

- A principle recorded in 2018 about how memory allocators behave under fragmentation may be as valid in 2026 as when it was written — possibly more so, because it has been confirmed in multiple projects over those eight years.
- A note recorded last week about a specific API behaviour may already be wrong if the library shipped a breaking change yesterday.

Mnemosyne uses **evidence-based evolution**. Knowledge is valid until it is contradicted by observation. The mechanisms that update knowledge are:

1. **Contradiction detection** at promotion time — when a new entry overlaps with existing ones
2. **Divergence detection** during curation — when recent project activity implicitly contradicts global knowledge
3. **Explicit supersession** during curation — when you deliberately update an entry because your understanding has changed

These mechanisms are all grounded in evidence: new observations that create tension with existing beliefs. The system surfaces these tensions and asks you to resolve them.

---

## Why Knowledge Should Not Auto-Expire

The temptation to auto-expire exists because old knowledge can be wrong, and stale entries in a knowledge base degrade its quality. But auto-expiry is a blunt instrument that creates two failure modes:

**False positives (wrongly expired):** Fundamental knowledge is discarded because it's old. Principles about concurrency, error propagation, memory management, and interface design often have long lifespans. Auto-expiry discards them because of age, not error.

**False negatives (not expired when it should be):** Knowledge that should be updated isn't, because the user has reviewed it within the time window even though nothing has changed. Time-based review does not guarantee correctness — it guarantees process.

The Mnemosyne approach trades administrative simplicity for epistemic honesty: knowledge is reviewed when evidence creates tension, not on a schedule. The `last_validated` field records when an entry was last deliberately confirmed, providing a weak signal about how recently it has been considered — but `last_validated` is used as context in curation, not as an expiry gate.

---

## Confidence Level Semantics

Confidence levels express epistemic state, not importance. An entry can be highly important but low-confidence (something you observed once and really need to understand better) or less important but high-confidence (a settled convention you've applied hundreds of times).

| Level | Meaning | Typical source |
|-------|---------|----------------|
| `high` | Validated across multiple projects or in a high-stakes context. You would stake a production deployment on it. | Multiple projects; high-impact events (incidents, post-mortems) |
| `medium` | Validated in one project, very likely broadly applicable. This is the default confidence for a successfully promoted learning. | Single-project promotion via `/reflect` |
| `low` | Observed once, tentative. Recorded because it may matter, not because it is certain. | First-time observations; uncertain situations |
| `prospective` | Awareness of a possibility, not yet validated by hands-on experience. | Horizon scanning; secondhand reports; documentation reading |

**Confidence is not automatically upgraded.** Upgrading from `low` to `medium`, or `medium` to `high`, requires deliberate confirmation during curation or a new promotion that reinforces the existing entry. Automation cannot make this judgement — only you can decide when evidence warrants increased confidence.

**Prospective entries are not lesser entries.** They represent important epistemic states: things you are aware of but have not yet worked with. They should be tracked, revisited, and either confirmed or discarded based on experience. The `explore` command surfaces them explicitly.

---

## Contradiction Detection

Contradiction detection fires when you promote a new entry and existing entries have overlapping tags. The underlying mechanism is Jaccard similarity.

### Jaccard Similarity

Given two tag sets A and B, the Jaccard similarity score is:

```
J(A, B) = |A ∩ B| / |A ∪ B|
```

For example:
- Entry A: `[rust, async, tokio, channels]`
- Entry B: `[rust, async, tokio, error-handling]`
- Intersection: `{rust, async, tokio}` (3 elements)
- Union: `{rust, async, tokio, channels, error-handling}` (5 elements)
- J = 3/5 = 0.60

The system uses a threshold of **0.5** (50%). Any existing entry with Jaccard similarity ≥ 0.5 is flagged as a potential contradiction.

### Why Jaccard Threshold 0.5?

A 0.5 threshold means the intersection is at least as large as the non-overlapping portion. This captures entries that are "about the same thing" at a tag level — they share more topics than they differ on. This is the right signal for contradiction: entries about the same topic cluster that disagree.

A lower threshold (e.g., 0.3) would generate too many false positives — flagging unrelated entries that share one or two generic tags like `rust` or `async`. A higher threshold (e.g., 0.7) would miss genuine contradictions between closely related entries.

### Contradiction ≠ Conflict

High tag overlap does not mean the entries conflict — it means they are about the same topic and should be reviewed for consistency. Common outcomes:

- **Complementary** — different aspects of the same topic. Resolve with `coexist`.
- **Reinforcing** — the new entry confirms the existing one. Consider discarding the new entry and validating the existing one instead.
- **Genuine contradiction** — the entries disagree. Resolve with `supersede` or `refine`.
- **Scope difference** — one entry is more specific than the other. Resolve with `coexist` and add scope notes.

---

## Supersession

Supersession is the mechanism for recording that an old piece of knowledge has been replaced. Mnemosyne's supersession model has three properties:

1. **Inline preservation** — superseded content is preserved in the file itself, not deleted or archived separately
2. **Date ranges** — the record shows when the old knowledge was created and when it was superseded
3. **Reason required** — you must provide a reason, which becomes part of the historical record

### Why Inline Preservation?

Deleting superseded content destroys history. Knowledge that was wrong in one context might be correct in another. Knowledge that was superseded by a library update might become relevant again after a later update. The historical record of what was believed and why it changed is often as valuable as the current belief.

The `## Superseded` section at the end of a file keeps the history accessible without polluting the current guidance. Readers see the current knowledge first; the historical record is available when needed.

### Supersession During Promotion

When you choose `s` (supersede) during `mnemosyne promote`, the CLI:
1. Saves your new entry to the appropriate axis file
2. Appends a `## Superseded` record to the existing contradicting entry with the old content, a date range, and a reason

The existing entry's `supersedes` list is not modified automatically — the old-entry-side supersession record is the inline `## Superseded` section.

### Supersession During Curation

When you choose `s` (supersede) during `mnemosyne curate`, the CLI:
1. Appends a `## Superseded` record to the current entry's body
2. Prompts you for new content (currently in the `refine` implementation path)
3. Updates `last_validated` to today

---

## Divergence Detection

Divergence detection addresses a different signal than contradiction detection. Rather than comparing a new entry directly with existing ones, divergence looks at patterns across recent project activity.

### The Mechanism

Divergence is detected by comparing:
- **Global entries** — existing entries in the knowledge store
- **Recent entries** — entries from the store that have origins from projects active in the last 90 days that are not listed as origins of the global entry

If a global entry has high tag overlap (Jaccard ≥ 0.5) with recent entries from projects that are NOT in the global entry's origins list, and those entries are from 2 or more distinct projects, a divergence flag is raised.

### What Divergence Means

Divergence signals that multiple projects have been doing something different from what the global knowledge recommends, without that difference having been explicitly recorded. It is an implicit signal, not a direct contradiction — the recent entries do not directly contradict the global entry, but they represent behaviour from a new set of projects.

This matters because:
- Your global knowledge may have been formed from a small, specific set of projects
- As you work across more projects, you naturally discover that the situation is more nuanced
- Divergence is how that accumulated experience surfaces without requiring you to manually track which projects did what

### The Project Threshold

The default divergence threshold is **2 projects**. A single diverging project might be an anomaly; two or more suggests a genuine pattern worth examining.

---

## Reflective Curation

Curation in Mnemosyne is modelled on the expert practice of deliberate knowledge consolidation. It is not a maintenance chore — it is an investment in the quality of your accumulated expertise.

### When to Curate

Run `mnemosyne curate`:
- After completing a major project or milestone
- When you sense your understanding in an area has shifted significantly
- When you want to deliberately consolidate recent learnings
- Before starting a new project in an area where your knowledge base is dense

Avoid curation as a reflex response to age. The system will surface divergence and low-confidence entries — trust it to tell you when curation is warranted rather than scheduling it on a calendar.

### The Reflective Stance

Approach curation as a Socratic dialogue with your past self. Each entry represents a belief you held at some point. Ask:
- Does this still hold in every project I've worked in since?
- Has my understanding of this area deepened or shifted?
- Is the entry as precise as it should be, or was it written in a moment of overgeneralisation?
- If I were advising a junior developer, would I give this guidance confidently?

The interactive session is designed to slow you down. Reviewing entries one at a time, with explicit choices for each, forces deliberate evaluation rather than a bulk "mark all validated" shortcut.

---

## The Socratic Exploration Model

The `explore` command is named for a specific mode of thinking: questioning what you know to discover what you don't. The session is structured around three types of inquiry:

### Gap Analysis

Where is your knowledge thin or absent? Gap analysis looks for:
- Languages you work in that have few entries (fewer than 3)
- Tags that appear in only one entry (breadth without depth)
- A low entry-to-project ratio (you've worked on many projects but promoted little)

Gaps are not failures — they are opportunities. Identifying them is the first step to filling them.

### Open Questions

What do you believe tentatively, or know about only theoretically? Open questions surfaces:
- `confidence: low` entries — things observed once, waiting for confirmation
- `confidence: prospective` entries — things read about but not yet worked with

These are the frontier of your knowledge. Exploring them may involve looking for opportunities to test them in practice, researching them more deeply, or simply prompting a conversation about your experience with an LLM.

### Horizon Scanning

Horizon scanning (future work) identifies new developments in your active ecosystems — library updates, emerging patterns, new tools — and creates `prospective` entries for them. This is how the knowledge base grows forward rather than only consolidating backward.

### The Generation Phase

After analysis, `explore` invites you to enter any topic and record what you know. This is the generative phase: turning tacit knowledge (what you know but haven't written down) into explicit knowledge (what the system can surface for you and for LLMs working in your context).

The entry is saved at a confidence level you choose. A just-confirmed insight from recent project work might be `medium` or `high`. A theoretical understanding of something you haven't implemented gets `low` or `prospective`.

---

## The Full Evolution Pathway

A complete knowledge lifecycle in Mnemosyne follows this path:

```
Raw observation (in plan 🟡🔴)
    ↓ /reflect
Per-project learning (knowledge/ file)
    ↓ mnemosyne promote
Global learning (confidence: medium)
    ↓ used in multiple projects without contradiction
Global learning (confidence: high)
    ↓ new evidence contradicts it
Superseded (inline record in ## Superseded)
    ↓ no longer applicable to any current work
Archived (in archive/ with reason)
```

Each transition is gated by human judgement and evidence. The system provides the signals (contradictions, divergence, gaps) and the mechanisms (promotion, supersession, archival). You make the calls.

This design reflects a core belief: knowledge management for expert practitioners is not a workflow problem to be automated — it is a cognitive practice to be supported.
