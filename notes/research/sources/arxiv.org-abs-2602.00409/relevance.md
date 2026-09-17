# Relevance — Are Coding Agents Generating Over-Mocked Tests? An Empirical Study

**Verdict:** relevant — large-scale empirical evidence that agent/LLM-authored tests over-use mocking relative to human tests, bearing directly on the test-double/mocking dimension of an optimal testing strategy.

## Salient sections

- "coding agents are more likely to modify tests and to add mocks to tests than non-coding agents" (abstract) → identifies a systematic difference in how LLM/agent-authored tests are constructed; over-mocking is a candidate failure mode to guard against.
- "36% of commits made by coding agents add mocks to tests, compared with 26% by non-agents" and "23% of commits made by coding agents add/change test files, compared with 13% by non-agents" → quantified gap in mocking and test-touching behavior between agents and humans.
- "68% of the repositories with agent test activity also contain agent mock activity" + "repositories created recently contain a higher proportion of test and mock commits made by agents" → mocking is pervasive in agent test activity and trending up — a strategy-level concern as agent-authored code grows.
- "tests with mocks may be potentially easier to generate automatically (but less effective at validating real interactions), and the need to include guidance on mocking practices in agent configuration files" → actionable implication: testing strategy for LLM-authored code should constrain mocking (e.g., via agent config/guidance) to preserve real-interaction validation.

## Evidence weight

- Study type: large-scale observational mining of 2025 commit history — 1.2M+ commits, 2,168 repositories, 48,563 agent commits, 169,361 test-modifying commits, 44,900 mock-adding commits. arXiv preprint (v1, Jan 2026); accepted at MSR 2026 — peer-reviewed.
- Scope/sample: TypeScript, JavaScript, Python only; agent identification (commit signatures/authors) not detailed in abstract.
- Caveats: correlational, not causal; measures *prevalence* of mocking, not test effectiveness directly — the "less effective at validating real interactions" claim is asserted, not measured in captured text. Only the arXiv abstract page was captured.
