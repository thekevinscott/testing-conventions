# Summary

- **Title:** Evaluating LLM-Based Test Generation Under Software Evolution
- **Authors:** Sabaat Haroon, Mohammad Taha Khan, Muhammad Ali Gulzar
- **URL:** https://arxiv.org/abs/2603.23443
- **Date:** Submitted 24 Mar 2026 (v1)
- **Venue:** No venue stated. Comments: "10 pages, 9 figures, 2 tables." Subjects: cs.SE; cs.AI.
- **Source type:** (arXiv preprint — abstract page only, full paper not captured)

## What the source claims

Large-scale empirical study of how LLM-generated tests respond to code changes. Central
claim: LLM test generation relies on surface-level cues rather than genuine reasoning about
program behavior, so it degrades under both semantic-altering and semantic-preserving code
changes and struggles to maintain regression awareness.

Verbatim quotes from the abstract:

> "it remains unclear whether these tests reflect genuine reasoning about program behavior or
> simply reproduce superficial patterns learned during training."

> "Using an automated mutation-driven framework, we analyze how generated tests react to
> semantic-altering changes (SAC) and semantic-preserving changes (SPC) across eight LLMs and
> 22,374 program variants."

> "LLMs achieve strong baseline results, reaching 79% line coverage and 76% branch coverage
> with fully passing test suites on the original programs."

> "Under SACs, the pass rate of newly generated tests drops to 66%, and branch coverage
> declines to 60%. More than 99% of failing SAC tests pass on the original program while
> executing the modified region, indicating residual alignment with the original behavior
> rather than adaptation to updated semantics."

> "Performance also declines under SPCs despite unchanged functionality: pass rates fall to
> 79% and branch coverage to 69%."

> "Although SPC edits preserve semantics, they often introduce larger syntactic changes,
> leading to instability in generated test suites. Models generate more new tests while
> discarding many baseline tests, suggesting sensitivity to lexical changes rather than true
> semantic impact."

> "Overall, our results indicate that current LLM-based test generation relies heavily on
> surface-level cues and struggles to maintain regression awareness as programs evolve."

## Method / evidence type

- "large-scale empirical study of LLM-based test generation under program changes."
- "automated mutation-driven framework."
- Two change types: semantic-altering changes (SAC) and semantic-preserving changes (SPC).
- Across "eight LLMs and 22,374 program variants."

## Numbers recorded (verbatim from abstract)

- Baseline (original programs): "79% line coverage and 76% branch coverage with fully passing
  test suites."
- Under SACs: "pass rate of newly generated tests drops to 66%, and branch coverage declines
  to 60%."
- "More than 99% of failing SAC tests pass on the original program while executing the
  modified region."
- Under SPCs: "pass rates fall to 79% and branch coverage to 69%."
- Scale: "eight LLMs and 22,374 program variants."

## Scope, limitations, and gaps (as observable from the abstract)

- Models not named in the abstract ("eight LLMs"); programming language(s) not stated.
- "Mutation-driven framework" generates program variants — relation to real developer code
  evolution not quantified in captured text.
- Coverage/pass-rate metrics reported; fault-detection (mutation-kill) outcomes not stated in
  abstract.
- Full methodology, model list, and per-model results are in the full paper, **not captured
  here** — only the arXiv abstract page was fetched.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
title, authors, full abstract, and bibliographic metadata, but **not** the full paper body
(PDF at `/pdf/2603.23443`, HTML at `/html/2603.23443v1` — not fetched). Paper is dated
March 2026.
