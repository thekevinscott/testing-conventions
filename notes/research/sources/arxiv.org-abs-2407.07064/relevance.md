# Relevance — Prompting Techniques for Secure Code Generation: A Systematic Investigation

**Verdict:** relevant (broad-inclusion) — empirical evidence that the way LLM code is generated (prompting strategy) measurably changes its defect/vulnerability rate, which is upstream context for what a testing strategy for LLM-authored code must catch.

## Salient sections

- RESULTS (abstract) — "observes a reduction in security weaknesses across the tested LLMs, especially after using an existing technique called Recursive Criticism and Improvement (RCI)" → self-criticism/iteration measurably lowers vulnerabilities; relevant because it shows a complementary lever (generation-side) to test-based gating for LLM-authored code quality.
- METHOD (abstract) — "a systematic literature review to identify the existing prompting techniques ... A subset ... evaluated on GPT-3, GPT-3.5, and GPT-4 ... an existing dataset consisting of 150 NL security-relevant code-generation prompts." → comparative empirical design across models on a fixed security-prompt dataset, the kind of measured comparison the goal favors.
- Scope note (summary) — magnitude of the reduction is not quantified in the captured abstract, and only GPT-family closed models are evaluated → limits how strongly the effect transfers to open-source LLM code.

## Evidence weight

Peer-reviewed empirical study (SLR + evaluation, accepted ACM TOSEM Feb 2025) — primary empirical evidence on generation-side vulnerability reduction; adjacent to (not directly about) test-strategy effectiveness, and only the abstract was captured so effect sizes are unverified here.
