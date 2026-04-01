---
name: explore-knowledge
description: Interactive knowledge exploration — gap analysis, horizon scanning, open questions. Usage: /explore-knowledge [mode]
---

You are running an interactive knowledge exploration session. Unlike curation (which reviews existing knowledge for validity), exploration actively grows the knowledge base by identifying gaps, researching new developments, and surfacing unresolved questions.

## 1. Check prerequisites

Verify `mnemosyne` is installed:

```bash
which mnemosyne
```

If not found, you can still run a manual exploration session using the modes below — you just won't be able to read the existing knowledge base automatically. Proceed with what's available.

If installed, load the current knowledge state:

```bash
mnemosyne status
```

## 2. Choose exploration mode

If the user has not specified a mode, present the options:

> Exploration modes:
> 1. **Gap analysis** — identify thin or missing areas relative to your active domains
> 2. **Horizon scanning** — research new developments in your domains
> 3. **Open questions** — review unresolved tensions and low-confidence entries
>
> Which would you like to explore? (or run all three)

## 3. Gap analysis

Examine the knowledge base (via `mnemosyne query --context` if available, or by asking the user about their recent work) to identify:

- Domains where the developer is active but knowledge is thin or absent
- Technologies used together where only one side is documented
- Techniques referenced in existing entries but without their own dedicated entry

For each gap, ask a targeted question: "You have entries about Rust async patterns but nothing about error handling in async contexts. What's your approach?" 

Distil the developer's responses into candidate knowledge entries with:
- Suggested title
- Appropriate tags
- Axis placement (language, domain, tool, or technique)
- Confidence level (high/medium/low based on how settled the developer's view seems)

Offer to promote candidates via `/promote-global` or note them for later.

## 4. Horizon scanning

Search for new developments in the developer's active domains. Look for:
- New libraries, frameworks, or tools in relevant ecosystems
- Architectural patterns gaining traction in relevant communities
- Breaking changes, deprecations, or security advisories affecting tools in the knowledge base
- Research or conference material related to the developer's technique areas

Use web search to find current information. Present findings as discussion points rather than assertions: "There's a new Rust error handling crate called `error-stack` that takes a different approach to the context pattern you've documented. Want to explore it?"

Record the developer's assessment as **prospective** knowledge — confidence level `prospective` indicates awareness of possibilities, not validated experience. These entries are clearly distinguished from experience-validated knowledge.

If `mnemosyne` is available, create prospective entries via:

```bash
mnemosyne promote --tags <tags> --origin global
```

Set confidence to `prospective` when prompted.

## 5. Open questions

Review unresolved tensions and low-confidence entries:

If `mnemosyne` is available, query for entries that warrant review:

```bash
mnemosyne query --context --format markdown
```

Look specifically for:
- Entries where the developer previously chose "coexist" during contradiction resolution — are those contexts still distinct?
- Entries with `confidence: low` or `confidence: prospective` — has experience clarified them?
- Clusters of related entries that might benefit from a synthesised overview

For each open question, engage the developer: "You have two entries about async channel patterns that you noted could coexist. After more experience, does one approach win, or do both still hold?" Update or supersede entries based on the discussion.

## 6. Summary

After the session, report:
- Gaps identified: N
- Candidate entries created: M
- Prospective entries added: K
- Open questions resolved: J

Suggest scheduling the next exploration session: "Consider running /explore-knowledge again after your next major project to capture new patterns."
