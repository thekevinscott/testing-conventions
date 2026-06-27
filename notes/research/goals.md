# Goal

Produce a **definition of the optimal testing strategy for LLM-authored open-source
code**, grounded in **empirical, peer-reviewed research**.

## Terms (atomic)

- **LLM-authored** — the code under test is written by a large language model, not a human.
- **Open-source** — publicly released, community-visible, community-maintained code.
- **Testing strategy** — the methodology for testing it: what to test, at what granularity,
  with which techniques (e.g. example-based vs property-based, mutation testing, coverage
  targets, test-doubles/mocking), and what gate decides a test (or the code) is acceptable.
- **Optimal** — best on a *measured* criterion (fault/bug detection, robustness, maintainability,
  etc.). "Optimal" claims must come from **comparative empirical evidence**, not assertion.
- **Grounded in empirical, peer-reviewed research** — the definition follows primary
  scientific studies. Not invented here; not based on opinion/blogs/vendor posts (those are
  context at most, never the evidentiary basis).

## Relevance rubric (for judging sources)

A source is **relevant** if it provides **empirical evidence** bearing on *how to test
LLM-authored code and which strategies measurably work best* — e.g. comparative studies of
testing techniques on LLM-generated code, measured effectiveness (fault detection, mutation
score, coverage), or documented failure modes of LLM-generated tests. Peer-reviewed primary
studies are the core; opinion/blog/vendor pieces are context, not the basis, and should be
marked as such.
