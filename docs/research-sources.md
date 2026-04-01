# Research Sources

Mnemosyne is grounded in cognitive science and knowledge management research. This document is an annotated bibliography of the key sources that have shaped the design, with notes on how each influenced the system.

## Contents

- [Observational Memory Systems](#observational-memory-systems)
- [Human Memory Models](#human-memory-models)
- [Belief Revision Theory](#belief-revision-theory)
- [Expertise Accumulation Research](#expertise-accumulation-research)
- [Spaced Retrieval and the Testing Effect](#spaced-retrieval-and-the-testing-effect)
- [The Zettelkasten Method](#the-zettelkasten-method)
- [Cognitive Load Theory](#cognitive-load-theory)
- [Knowledge Management in Software Engineering](#knowledge-management-in-software-engineering)
- [Further Reading](#further-reading)

---

## Observational Memory Systems

### Mastra Observational Memory

**Mastra** is an open-source TypeScript AI agent framework that introduced the observational memory pattern for LLM-driven development. The core insight is that LLM agents working on software projects produce observations — incidental discoveries about the codebase, environment, and constraints — that should be promoted to a structured knowledge base to survive across sessions.

Mastra's approach formalises the Do → Verify → Observe cycle: for each implementation step, do the work, verify it works, and record what you learned. Observations are graded by priority (critical, useful, informational) and promoted to a knowledge base indexed by project structure.

**Influence on Mnemosyne:** The per-project Tier 1 of Mnemosyne is a direct implementation of Mastra's observational memory concepts. The observation priority codes (🔴🟡🟢⚪), the `/reflect` promotion cycle, and the plan format with `## Observations` and `## Promoted` sections are all drawn from this model. The key Mnemosyne contribution is extending this per-project model to a global, cross-project Tier 2 with evidence-based evolution.

---

## Human Memory Models

### Distinction Between Episodic and Semantic Memory

**Tulving, E. (1972).** Episodic and semantic memory. In E. Tulving & W. Donaldson (Eds.), *Organisation of Memory*. Academic Press.

Tulving's foundational distinction separates two memory systems: *episodic memory* (memory of specific events — what happened, when, in what context) and *semantic memory* (general knowledge about the world, independent of the specific episode where it was learned).

Expert practitioners naturally consolidate episodic memories into semantic knowledge over time. "I remember the specific incident where unbounded channels caused memory exhaustion in the api-server project in 2026" (episodic) becomes "unbounded channels cause memory exhaustion under sustained load" (semantic).

**Influence on Mnemosyne:** The `origins` field in knowledge entries preserves the episodic provenance (which project, when, what context) while the entry title and body express semantic knowledge (the transferable insight). The promotion pathway — from plan observation to per-project learning to global knowledge — mirrors the episodic-to-semantic consolidation process. The `last_validated` field represents semantic validation: confirming that the generalisation still holds.

---

### The Consolidation Process

**McClelland, J.L., McNaughton, B.L., & O'Reilly, R.C. (1995).** Why there are complementary learning systems in the hippocampus and neocortex: Insights from the successes and failures of connectionist models of learning and memory. *Psychological Review*, 102(3), 419–457.

This paper proposes the complementary learning systems (CLS) theory: fast, plastic learning in the hippocampus captures specific episodes, while slow, distributed learning in the neocortex builds stable semantic representations. Sleep replay gradually integrates episodic memories into semantic knowledge.

**Influence on Mnemosyne:** The two-tier architecture mirrors CLS theory. Per-project knowledge (Tier 1) is like hippocampal storage — fast, specific, episodic. Global knowledge (Tier 2) is like neocortical storage — stable, general, semantic. The promotion ceremony (promotion from per-project to global during `/reflect`) is the system's version of memory consolidation. This is why promotion is not automatic: it requires deliberate review, analogous to the active consolidation process.

---

## Belief Revision Theory

### AGM Belief Revision

**Alchourrón, C.E., Gärdenfors, P., & Makinson, D. (1985).** On the logic of theory change: Partial meet contraction and revision functions. *Journal of Symbolic Logic*, 50(2), 510–530.

The AGM framework is the foundational formal theory of how a rational agent should update beliefs in the face of new information. It defines three types of change: *expansion* (adding new information consistent with existing beliefs), *contraction* (removing a belief when evidence suggests it is false), and *revision* (adding new information that contradicts existing beliefs, requiring some existing beliefs to be retracted to maintain consistency).

AGM's rationality postulates require that revision minimise change: when incorporating new information, retain as much existing belief as possible. This is the principle of *minimal change* or *conservative revision*.

**Influence on Mnemosyne:** The contradiction detection and resolution workflow is an application of AGM belief revision. When a new entry contradicts an existing one, the `[s]upersede / [c]oexist / [d]iscard / [r]efine` choices correspond to the three types of change: supersede is revision (new replaces old), coexist is expansion (both can be true), discard is contraction (new information rejected), refine is a nuanced revision. The supersession record honours minimal change by preserving the old content inline rather than deleting it.

---

## Expertise Accumulation Research

### Deliberate Practice and the Development of Expertise

**Ericsson, K.A., Krampe, R.T., & Tesch-Römer, C. (1993).** The role of deliberate practice in the acquisition of expert performance. *Psychological Review*, 100(3), 363–406.

Ericsson's deliberate practice framework identifies the key features of expertise-building activities: focused effort on performance just beyond current competence, immediate feedback, and structured reflection on results. Mere repetition without reflection does not build expertise — deliberate engagement with what went wrong, why, and how to improve is essential.

**Influence on Mnemosyne:** The curation session is designed as deliberate practice. Rather than passively accumulating knowledge entries, the `curate` command asks you to actively evaluate each entry: does it still hold? Has your understanding deepened? This reflective engagement — requiring explicit confirmation or revision rather than passive review — mirrors the deliberate practice framework.

---

### Expert Knowledge as Chunking

**Chase, W.G., & Simon, H.A. (1973).** Perception in chess. *Cognitive Psychology*, 4(1), 55–81.

Chase and Simon's landmark study showed that chess masters recognise roughly 50,000 meaningful "chunks" — patterns of pieces they have seen and understood in context — compared to a few hundred for novices. Expert performance relies not on faster computation but on a much richer repertoire of meaningful patterns.

**Influence on Mnemosyne:** Each knowledge entry is a chunk: a recognised pattern with associated meaning, constraints, and implications. The axis organisation (languages, domains, tools, techniques) corresponds to the taxonomies experts use to organise chunks. The tag system enables the cross-referencing that makes chunks useful across contexts, mirroring how expert knowledge is not isolated facts but a web of related patterns.

---

## Spaced Retrieval and the Testing Effect

### The Testing Effect

**Roediger, H.L., & Karpicke, J.D. (2006).** Test-enhanced learning: Taking memory tests improves long-term retention. *Psychological Science*, 17(3), 249–255.

Retrieval practice — the act of retrieving information from memory — is more effective for long-term retention than re-studying the same material. The "testing effect" is robust across many domains and memory types.

**Influence on Mnemosyne:** The `query` command is a form of retrieval practice. When you query global knowledge at the start of a session, you are retrieving relevant entries — reinforcing them. The `last_validated` field records when an entry was last retrieved and confirmed, providing weak spaced retrieval scheduling signals during curation.

---

### Spaced Repetition

**Ebbinghaus, H. (1885/1913).** *Memory: A Contribution to Experimental Psychology*. Teachers College, Columbia University.

Ebbinghaus's forgetting curve describes the exponential decay of memory over time. Spaced repetition systems (Anki, Supermemo) exploit the spacing effect: reviewing material at increasing intervals, just before it would be forgotten, is far more efficient than massed practice.

**Influence on Mnemosyne (by contrast):** Mnemosyne deliberately does not implement spaced repetition for knowledge maintenance. The reason: spaced repetition is designed for the retrieval of specific facts (vocabulary, formulae). Expert developer knowledge is not facts to be recalled but models to be applied. A principle about connection pooling does not decay through non-use — it remains valid until the libraries change. Forcing spaced review of stable knowledge wastes effort; reviewing unstable knowledge (via divergence detection) focuses effort where it matters.

---

## The Zettelkasten Method

### Luhmann's Zettelkasten

**Luhmann, N. (1981).** Kommunikation mit Zettelkästen. In H. Baier, H.M. Kepplinger, & K. Reumann (Eds.), *Öffentliche Meinung und sozialer Wandel*. Westdeutscher Verlag.

Niklas Luhmann, the prolific sociologist, used a card-based knowledge management system (Zettelkasten, or "slip box") to build a web of 90,000 interconnected ideas over 40 years. The key insight is that knowledge grows not just by adding cards but by creating links: each new card is connected to existing cards, creating emergent structures that the creator did not explicitly design.

**Influence on Mnemosyne:** The tag-based cross-referencing system is inspired by Zettelkasten linking. Tags serve as the Mnemosyne equivalent of Luhmann's reference numbers: they connect entries across axis boundaries, enabling emergent cross-domain knowledge clusters to form naturally over time. The `supersedes` field in frontmatter is a direct link, analogous to a Zettelkasten reference to a card this one revises.

---

### Building a Second Brain

**Forte, T. (2022).** *Building a Second Brain: A Proven Method to Organise Your Digital Life and Unlock Your Creative Potential*. Profile Books.

Forte's "second brain" methodology (also PARA — Projects, Areas, Resources, Archives) organises personal knowledge around the actionability of information. The insight most relevant to Mnemosyne is the CODE framework: Capture, Organise, Distil, Express.

**Influence on Mnemosyne:** The promotion pathway (capture via plan observations → organise by axis → distil during curation → express via query) directly mirrors the CODE cycle. The `archive/` directory in Mnemosyne corresponds to Forte's Archive: material that is no longer active but preserved for potential future relevance.

---

## Cognitive Load Theory

### Working Memory and Extraneous Load

**Sweller, J. (1988).** Cognitive load during problem solving: Effects on learning. *Cognitive Science*, 12(2), 257–285.

**Sweller, J., van Merriënboer, J.J.G., & Paas, F.G.W.C. (1998).** Cognitive architecture and instructional design. *Educational Psychology Review*, 10(3), 251–296.

Cognitive load theory distinguishes intrinsic load (the complexity of the material itself), extraneous load (load imposed by poor presentation or unnecessary complexity), and germane load (load that contributes to schema formation). Effective learning maximises germane load while minimising extraneous load.

**Influence on Mnemosyne:** The knowledge format — title, tags, dated entries, priority codes — is designed to minimise extraneous cognitive load when the LLM or developer processes it. The axis hierarchy provides a pre-organised schema that reduces the mental effort of locating relevant knowledge. The `--max-tokens` flag on `query` prevents context overload: too much knowledge injected at once overwhelms rather than helps, similar to extraneous cognitive load.

---

## Knowledge Management in Software Engineering

### Tacit Knowledge in Software Development

**Tautz, C., & Althoff, K.D. (1997).** Using case-based reasoning for reusing software knowledge. *Proceedings of the Second International Conference on Case-Based Reasoning*, 156–165.

Tautz and Althoff document the challenge of capturing and reusing tacit knowledge in software engineering. Unlike explicit knowledge (documented procedures, specifications), tacit knowledge consists of heuristics, pattern recognition, and contextual judgement built through experience. Making tacit knowledge explicit without losing fidelity is one of the hardest problems in software knowledge management.

**Influence on Mnemosyne:** The observation-to-knowledge promotion cycle is designed to externalise tacit knowledge at the moment it surfaces — during active implementation, when the developer has just encountered the situation that activated the knowledge. The plan format's `## Observations` section captures this in-context, while the `## Promoted` section records what was successfully externalised. The quality of the externalisation depends on the developer's articulation; Mnemosyne provides the structure and prompts, not the words.

---

### Lessons Learned Systems in Software Projects

**Birk, A., Dingsøyr, T., & Stålhane, T. (2002).** Postmortem: Never leave a project without it. *IEEE Software*, 19(3), 43–45.

Birk et al. survey "lessons learned" systems in software organisations and identify why they so frequently fail to deliver value: lessons are captured too late (after the project), too abstractly (without the context that makes them actionable), or too rarely retrieved (the system exists but is not consulted). They recommend capture during projects, at the level of specificity needed to apply the lesson, with explicit retrieval mechanisms.

**Influence on Mnemosyne:** This research directly motivates several Mnemosyne design decisions. Lessons are captured during projects (via plan observations) rather than in post-mortems. The `origins` provenance preserves the specific context that makes lessons actionable. Context-inferred query (`--context`) makes retrieval automatic at session start, solving the "rarely consulted" failure mode. Confidence levels address the abstraction problem by marking the degree of generalisation explicitly.

---

## Further Reading

**Weinberg, G.M. (1971).** *The Psychology of Computer Programming*. Van Nostrand Reinhold.

An early and still-relevant exploration of how programmers think, communicate, and accumulate expertise. Weinberg's concept of "egoless programming" and his observations about how programmers learn from each other (or fail to) predate the knowledge management literature but are highly consistent with it.

---

**Dreyfus, H.L., & Dreyfus, S.E. (1986).** *Mind over Machine: The Power of Human Intuition and Expertise in the Era of the Computer*. Free Press.

The Dreyfus model of skill acquisition — novice, advanced beginner, competent, proficient, expert — describes how practitioners move from rule-following to pattern recognition to intuitive judgement. Expert knowledge is characterised by holistic recognition: the expert does not consult rules, they recognise situations. Mnemosyne's knowledge entries attempt to capture the rules and patterns that novices and intermediates need, while acknowledging that expert practice transcends them.

---

**Norman, D.A. (2013).** *The Design of Everyday Things* (revised edition). Basic Books.

Norman's concept of affordances — the properties of an object that suggest how it can be used — informs the Mnemosyne knowledge format design. Each frontmatter field, body section convention, and priority code is designed to make its intended use self-evident. An entry should "afford" retrieval, validation, and evolution without requiring the user to remember a procedure.
