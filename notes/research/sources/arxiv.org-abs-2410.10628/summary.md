# Summary

- **Title:** Test smells in LLM-Generated Unit Tests
- **Authors:** Wendkûuni C. Ouédraogo, Yinghua Li, Xueqi Dang, Xunzhu Tang, Anil Koyuncu, Jacques Klein, David Lo, Tegawendé F. Bissyandé
- **URL:** https://arxiv.org/abs/2410.10628
- **Date:** Submitted on 14 Oct 2024 (v1); last revised 6 Nov 2025 (v2, this version)
- **Venue:** Not stated on the abstract page
- **Source type:** (arXiv preprint — abstract page only, full paper not captured)

## What the source claims

Presents what it calls the first multi-benchmark, large-scale analysis of "test smell" diffusion in
LLM-generated unit tests, contrasting LLM outputs with human-written suites and with EvoSuite
(SBST) tests. Finds LLM-generated tests consistently manifest smells such as Assertion Roulette and
Magic Number Test, with patterns shaped by prompting strategy, context length, and model scale,
and overlaps with human-written tests that raise potential data-leakage concerns.

Verbatim quotes from the abstract:

> "This paper presents the first multi-benchmark, large-scale analysis of test smell diffusion in
> LLM-generated unit tests."

> "We contrast LLM outputs with human-written suites (as the reference for real-world practices)
> and SBST-generated tests from EvoSuite (as the automated baseline), disentangling whether LLMs
> reproduce human-like flaws or artifacts of synthetic generation."

> "Our study draws on 20,505 class-level suites from four LLMs (GPT-3.5, GPT-4, Mistral 7B, Mixtral
> 8x7B), 972 method-level cases from TestBench, 14,469 EvoSuite tests, and 779,585 human-written
> tests from 34,635 open-source Java projects."

> "Using two complementary detection tools (TsDetect and JNose), we analyze prevalence,
> co-occurrence, and correlations with software attributes and generation parameters."

> "Results show that LLM-generated tests consistently manifest smells such as Assertion Roulette and
> Magic Number Test, with patterns strongly influenced by prompting strategy, context length, and
> model scale."

> "Comparisons reveal overlaps with human-written tests, raising concerns of potential data leakage
> from training corpora while EvoSuite exhibits distinct, generator-specific flaws."

> "These findings highlight both the promise and the risks of LLM-based test generation, and call
> for the design of smell-aware generation frameworks, prompt engineering strategies, and enhanced
> detection tools to ensure maintainable, high-quality test code."

## Method / evidence type

- Large-scale empirical analysis of test smells across multiple benchmarks (Java).
- Comparison groups: four LLMs, EvoSuite (SBST baseline), and human-written tests.
- Detection tools: "two complementary detection tools (TsDetect and JNose)."
- Analyses: prevalence, co-occurrence, and correlations with software attributes and generation
  parameters.

## Numbers recorded

- "20,505 class-level suites from four LLMs (GPT-3.5, GPT-4, Mistral 7B, Mixtral 8x7B)."
- "972 method-level cases from TestBench."
- "14,469 EvoSuite tests."
- "779,585 human-written tests from 34,635 open-source Java projects."
- Named smells: "Assertion Roulette and Magic Number Test."
- (Metadata only: v1 submission size 696 KB; v2 size 1,609 KB.)

## Scope, limitations, and gaps (as observable from the abstract)

- Java-only across all comparison groups; cross-language generalization not addressed in the abstract.
- Four LLMs (GPT-3.5, GPT-4, Mistral 7B, Mixtral 8x7B); no claim about other model families.
- Data-leakage concern is raised as an inference from overlap with human-written tests, not a direct
  measurement, per the abstract.
- Prevalence magnitudes, per-smell rates, and correlation strengths are not quantified in the
  captured text — only counts of the corpora analyzed.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
title, authors, full abstract, version history, and bibliographic metadata, but **not** the full
paper body (PDF at `/pdf/2410.10628`, HTML at `/html/2410.10628v2` — not fetched).
