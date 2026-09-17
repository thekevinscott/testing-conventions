# Relevance — A Comprehensive Study on Large Language Models for Mutation Testing

**Verdict:** relevant — comprehensive empirical comparison of LLM-based vs rule-based mutant generation on 851 real Java bugs, quantifying fault-detection effectiveness and its cost — a general SE study of mutation-testing effectiveness that informs what gate a test should clear.

## Salient sections

- Headline effectiveness: "compared to existing rule-based approaches, LLMs generate more diverse mutants, that are behaviorally closer to real bugs and, most importantly, with 111.29% higher fault detection. That is, 87.98% (for LLMs) vs. 41.64% (for rule-based); an increase of 46.34 percentage points." → Mutation testing whose mutants better approximate *real* faults nearly doubles fault detection; this supports "tests must kill realistic mutants" as a strong acceptance gate for a test suite (including suites guarding LLM-authored code).
- Cost tradeoff: "these impressive results ... come at a cost: the LLM-generated mutants have worse non-compilability, duplication, and equivalent mutant rates by 26.60, 10.14, and 3.51 percentage points, respectively." → An optimal mutation-based gate must add filtering for non-compilable, duplicate, and equivalent mutants, or the effectiveness gain is diluted by wasted/uninformative mutants.
- Scope and baselines: "851 real bugs from two Java real-world bug benchmarks" and "seven LLMs using our newly designed prompt, including both leading open- and closed-source models," vs BugFarm and LLMorpheus. → Provides a state-of-the-art baseline; the use of *real* bugs (not synthetic) strengthens external validity of the fault-detection numbers.
- Actionability claim: findings "allow practitioners to have greater confidence in deploying LLM-based mutation." → Endorses LLM-driven mutation testing as a measurably stronger fault-injection method for test adequacy.

## Evidence weight

- **Study type:** Comprehensive empirical comparison study (arXiv preprint, v5; venue not stated on abstract page).
- **Scope/sample:** 851 real bugs across two Java real-world benchmarks; seven LLMs (open- and closed-source); two SOTA LLM baselines plus rule-based approaches.
- **Caveats:** Java-only (no cross-language evidence); the seven LLMs and the two benchmarks are not individually named in the abstract; no per-model breakdown, statistical-significance testing, or variance reported in the captured text; five versions exist (v1 2024 → v5 2026) and the abstract reflects v5. Strong, real-bug-grounded evidence for mutation as a fault proxy.
- **Capture caveat:** Judgment rests on the captured abstract + metadata; full paper body not fetched.
