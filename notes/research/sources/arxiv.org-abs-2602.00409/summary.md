# Summary

- **Title:** Are Coding Agents Generating Over-Mocked Tests? An Empirical Study
- **Authors:** Andre Hora, Romain Robbes
- **URL:** https://arxiv.org/abs/2602.00409
- **Date:** Submitted 30 Jan 2026 (v1)
- **Venue:** "Accepted for publication at MSR 2026" (Mining Software Repositories). Subjects: Software Engineering (cs.SE).
- **Source type:** (arXiv preprint; conference acceptance noted — abstract page only, full paper not captured)

## What the source claims

First study of mocking in agent-generated tests of real-world repositories. Central claim:
coding agents are more likely than non-agents to modify tests and to add mocks to tests,
which may make tests easier to generate but less effective at validating real interactions.

Verbatim quotes from the abstract:

> "In particular, excessive use of mocking can make tests harder to understand and maintain.
> This paper presents the first study to investigate the presence of mocks in agent-generated
> tests of real-world software systems."

> "We analyzed over 1.2 million commits made in 2025 in 2,168 TypeScript, JavaScript, and
> Python repositories, including 48,563 commits by coding agents, 169,361 commits that modify
> tests, and 44,900 commits that add mocks to tests."

> "Overall, we find that coding agents are more likely to modify tests and to add mocks to
> tests than non-coding agents."

> "We detect that (1) 60% of the repositories with agent activity also contain agent test
> activity; (2) 23% of commits made by coding agents add/change test files, compared with 13%
> by non-agents; (3) 68% of the repositories with agent test activity also contain agent mock
> activity; (4) 36% of commits made by coding agents add mocks to tests, compared with 26% by
> non-agents; and (5) repositories created recently contain a higher proportion of test and
> mock commits made by agents."

> "We call attention to the fact that tests with mocks may be potentially easier to generate
> automatically (but less effective at validating real interactions), and the need to include
> guidance on mocking practices in agent configuration files."

## Method / evidence type

- Large-scale mining of real-world repository commit history (2025 commits).
- Languages: TypeScript, JavaScript, and Python.
- Comparison of coding-agent vs non-coding-agent behavior on test modification and mock
  addition.
- Cross-cutting analysis by repository creation recency.

## Numbers recorded (verbatim from abstract)

- "over 1.2 million commits made in 2025 in 2,168 TypeScript, JavaScript, and Python
  repositories."
- "48,563 commits by coding agents."
- "169,361 commits that modify tests."
- "44,900 commits that add mocks to tests."
- "(1) 60% of the repositories with agent activity also contain agent test activity."
- "(2) 23% of commits made by coding agents add/change test files, compared with 13% by
  non-agents."
- "(3) 68% of the repositories with agent test activity also contain agent mock activity."
- "(4) 36% of commits made by coding agents add mocks to tests, compared with 26% by
  non-agents."
- "(5) repositories created recently contain a higher proportion of test and mock commits
  made by agents."

## Scope, limitations, and gaps (as observable from the abstract)

- Observational/correlational design — measures commit-level association between agent
  authorship and mocking, not causation or test quality directly.
- "Over-mocked" is framed as a concern; the abstract asserts mocks are "less effective at
  validating real interactions" but does not report a direct measurement of test
  effectiveness in the captured text.
- How "coding agents" are identified (commit signatures/authors) is not detailed in the
  abstract.
- Three languages only; generalisation to other ecosystems not addressed in captured text.
- Full methodology, detection criteria, and per-language breakdowns are in the full paper,
  **not captured here** — only the arXiv abstract page was fetched.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
title, authors, full abstract, and bibliographic metadata, but **not** the full paper body
(PDF at `/pdf/2602.00409`, HTML at `/html/2602.00409v1` — not fetched). Paper is dated
January 2026.
