# Relevance — LLMs Cannot Self-Correct Reasoning Yet

**Verdict:** relevant (broad-inclusion, thin tie) — indirect support for requiring an *external* verification oracle in a testing strategy for LLM code, since the paper shows LLMs cannot reliably correct themselves without external feedback.

## Salient sections
- Core finding (abstract, quoted in summary): "LLMs struggle to self-correct their responses without external feedback, and at times, their performance even degrades after self-correction." → Argues an LLM cannot be trusted to validate or fix its own output unaided, so the testing strategy needs an independent oracle (real tests/external checks), not model self-review.
- Scope of "intrinsic self-correction" — revising "based solely on its inherent capabilities, without the crutch of external feedback." → By contrast, tests are exactly the external feedback the paper says is needed; this frames testing as the missing verifier.

## Evidence weight
Primary empirical evidence, but the connection is thin: the study is about *reasoning* tasks and *intrinsic* self-correction, not code or code testing, and it does not evaluate any testing technique. It supports the general principle of external verification by analogy only — non-direct context, not evidence for an optimal testing strategy. Only the abstract was captured; models, benchmarks, and magnitudes are in the uncaptured full paper.
