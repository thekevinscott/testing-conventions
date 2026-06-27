# Summary

- **Title:** LLMs taking shortcuts in test generation: A study with SAP HANA and LevelDB
- **Authors:** Vekil Bekmyradov, Noah C. Pütz, Thomas Bartz-Beielstein
- **URL:** https://arxiv.org/abs/2604.14437
- **Date:** Submitted 15 Apr 2026 (v1)
- **Venue:** No peer-reviewed venue stated. Report number: "THK-AI Research Report 2/2026." Subjects: cs.SE; cs.AI. MSC: 68T07; ACM: I.2.7, I.2.1, I.2.5.
- **Source type:** (arXiv preprint / institutional research report — abstract page only, full paper not captured)

## What the source claims

Investigates whether LLMs genuinely reason in automated test generation or take shortcuts,
by contrasting an open-source system (LevelDB) with a proprietary commercial codebase (SAP
HANA) guaranteed absent from training data. Central claim: LLMs excel on familiar
open-source benchmarks but struggle on unseen complex domains, prioritizing compilability
over semantic effectiveness — independent SE evidence that current LLMs lack robust
reasoning.

Verbatim quotes from the abstract:

> "recent research in cognitive science reveals that these models sometimes rely on shallow
> heuristics and memorization, taking shortcuts rather than demonstrating genuine cognitive
> abilities."

> "contrasting performance on an open-source system (LevelDB) with SAP HANA, one of the most
> widely deployed commercial database systems worldwide, whose proprietary codebase is
> guaranteed to be absent from training data."

> "We combine cognitive evaluation principles, drawing on Mitchell's mechanism-focused
> assessment methodology, with empirical software testing, employing mutation score and
> iterative compiler-feedback repair loops to assess both accuracy and underlying reasoning
> strategies."

> "Results show that LLMs excel on familiar, open-source benchmarks but struggle with unseen,
> complex domains, often prioritizing compilability over semantic effectiveness."

> "These findings provide independent software engineering evidence for the broader claim
> that current LLMs lack robust reasoning, and highlight the need for evaluation frameworks
> that penalize trivial shortcuts and reward true generalization."

## Method / evidence type

- Contrastive case study: open-source LevelDB vs proprietary SAP HANA (held out from training
  data by construction).
- "cognitive evaluation principles, drawing on Mitchell's mechanism-focused assessment
  methodology."
- Empirical software-testing metrics: "mutation score and iterative compiler-feedback repair
  loops."
- Assesses both accuracy and "underlying reasoning strategies."

## Numbers recorded (verbatim from abstract)

- None. The abstract reports qualitative findings only; no quantitative results (mutation
  scores, pass rates, sample sizes) are given in the captured abstract text.

## Scope, limitations, and gaps (as observable from the abstract)

- Two systems only (LevelDB, SAP HANA); generalisation beyond databases not addressed in
  captured text.
- No numbers in the abstract — magnitude of the open-source vs proprietary gap is not
  quantified here.
- LLM model(s) used are not named in the abstract.
- "Guaranteed to be absent from training data" is asserted (proprietary codebase) but the
  basis for the guarantee is not detailed in captured text.
- Full methodology and quantitative results are in the full paper, **not captured here** —
  only the arXiv abstract page was fetched. Small submission size (39 KB) noted on page.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
title, authors, full abstract, and bibliographic metadata, but **not** the full paper body
(PDF at `/pdf/2604.14437`, HTML at `/html/2604.14437v1` — not fetched). Paper is dated
April 2026.
