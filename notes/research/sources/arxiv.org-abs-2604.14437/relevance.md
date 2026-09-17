# Relevance — LLMs taking shortcuts in test generation: A study with SAP HANA and LevelDB

**Verdict:** relevant — empirical SE evidence that LLM test-generation quality collapses on code absent from training data, prioritizing compilability over fault-detecting (semantic) effectiveness; uses mutation score as the effectiveness metric.

## Salient sections

- "contrasting performance on an open-source system (LevelDB) with SAP HANA ... whose proprietary codebase is guaranteed to be absent from training data" → isolates a data-contamination confound; tests whether benchmark success generalizes to genuinely unseen LLM-targeted code (the open-source case is the goal's exact subject).
- "LLMs excel on familiar, open-source benchmarks but struggle with unseen, complex domains, often prioritizing compilability over semantic effectiveness" → a testing strategy for LLM-authored code must not trust that generated tests are fault-revealing just because they compile/pass; semantic (mutation-based) gates are needed.
- "employing mutation score and iterative compiler-feedback repair loops to assess both accuracy and underlying reasoning strategies" → mutation score used as the effectiveness measure; compiler-feedback repair loops examined as a generation strategy — both directly inform how to evaluate/produce LLM tests.
- "independent software engineering evidence for the broader claim that current LLMs lack robust reasoning, and highlight the need for evaluation frameworks that penalize trivial shortcuts and reward true generalization" → argues acceptance gates should penalize shortcut/compilability-only tests.

## Evidence weight

- Study type: contrastive empirical case study (LevelDB vs SAP HANA) combining cognitive-evaluation principles with mutation score and compiler-feedback repair loops. arXiv preprint / institutional report ("THK-AI Research Report 2/2026"); no peer-reviewed venue stated.
- Scope/sample: two systems only (both databases); generalization beyond databases not addressed. LLM model(s) not named in abstract.
- Caveats: abstract reports qualitative findings only — no mutation scores, pass rates, or sample sizes given in captured text, so the magnitude of the open-source-vs-proprietary gap is unquantified here. "Guaranteed absent from training data" is asserted, not evidenced, in the abstract. Only the arXiv abstract page was captured (small 39 KB submission noted).
