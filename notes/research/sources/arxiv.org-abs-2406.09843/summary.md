# Summary

- **Title:** A Comprehensive Study on Large Language Models for Mutation Testing
- **Authors:** Bo Wang, Mingda Chen, Ming Deng, Youfang Lin, Mark Harman, Mike Papadakis, Jie M. Zhang
- **URL:** https://arxiv.org/abs/2406.09843
- **Date:** Submitted on 14 Jun 2024 (v1); last revised 22 Jan 2026 (v5, this version)
- **Venue:** Not stated on the abstract page (39 pages, 4 figures; ACM class D.2.5)
- **Source type:** (arXiv preprint — abstract page only, full paper not captured)

## What the source claims

Empirical study comparing LLM-based mutant generation against rule-based approaches and against
two state-of-the-art LLM-based methods (BugFarm and LLMorpheus), evaluated on real bugs from Java
benchmarks. Central finding: LLMs generate more diverse mutants that are behaviorally closer to
real bugs and have much higher fault detection, but at the cost of worse non-compilability,
duplication, and equivalent-mutant rates.

Verbatim quotes from the abstract:

> "we conduct a comprehensive empirical study evaluating BugFarm and LLMorpheus (the two
> state-of-the-art LLM-based approaches), alongside seven LLMs using our newly designed prompt,
> including both leading open- and closed-source models, on 851 real bugs from two Java real-world
> bug benchmarks."

> "Our results reveal that, compared to existing rule-based approaches, LLMs generate more diverse
> mutants, that are behaviorally closer to real bugs and, most importantly, with 111.29% higher
> fault detection. That is, 87.98% (for LLMs) vs. 41.64% (for rule-based); an increase of 46.34
> percentage points."

> "Nevertheless, our results also reveal that these impressive results for improved effectiveness
> come at a cost: the LLM-generated mutants have worse non-compilability, duplication, and
> equivalent mutant rates by 26.60, 10.14, and 3.51 percentage points, respectively."

> "These findings are immediately actionable for both research and practice. They allow
> practitioners to have greater confidence in deploying LLM-based mutation, while researchers now
> have a baseline for the state-of-the-art, with which they can research techniques to further
> improve effectiveness and reduce cost."

## Method / evidence type

- Comprehensive empirical comparison study.
- Subjects: "851 real bugs from two Java real-world bug benchmarks."
- Compared systems: BugFarm, LLMorpheus, plus "seven LLMs using our newly designed prompt"
  (leading open- and closed-source models), versus rule-based approaches.
- Metrics: mutant diversity, behavioral closeness to real bugs, fault detection, non-compilability,
  duplication, and equivalent mutant rates.

## Numbers recorded

- "111.29% higher fault detection" for LLMs vs. rule-based.
- Fault detection: "87.98% (for LLMs) vs. 41.64% (for rule-based); an increase of 46.34 percentage
  points."
- LLM-generated mutants have "worse non-compilability, duplication, and equivalent mutant rates by
  26.60, 10.14, and 3.51 percentage points, respectively."
- Corpus: "851 real bugs from two Java real-world bug benchmarks"; "seven LLMs."
- (Metadata only: "39 pages, 4 figures"; v5 submission size 2,813 KB.)

## Scope, limitations, and gaps (as observable from the abstract)

- Java-only benchmarks; cross-language generalization not addressed in the abstract.
- The seven LLMs and the two benchmarks are not individually named in the captured text.
- The abstract states a tradeoff (effectiveness vs. cost) but no per-model breakdown, statistical
  testing, or variance figures.
- Five versions exist (v1 Jun 2024 through v5 Jan 2026); the captured abstract reflects v5 and may
  differ from earlier versions. Full methodology is in the paper body, not captured.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
title, authors, full abstract, version history, and bibliographic metadata, but **not** the full
paper body (PDF at `/pdf/2406.09843`, HTML at `/html/2406.09843v5` — not fetched).
