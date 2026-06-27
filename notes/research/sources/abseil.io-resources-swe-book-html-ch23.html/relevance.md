# Relevance — SWE at Google ch.23: Continuous Integration

**Verdict:** relevant (broad-inclusion) — describes the CI gate and continuous-testing structure (presubmit fast/reliable tests, progressively larger-scope post-submit, flaky-test management, hermetic testing) that frames *when* and *how* a testing strategy for LLM code runs and what gate accepts a change.

## Salient sections
- Fast-feedback rationale ("the cost of a bug grows almost exponentially the later it is caught") and the rule to run "only fast, reliable ones" (small/unit tests) on presubmit, catching the rest post-submit. → For LLM-authored code, supports a layered gate: cheap deterministic tests block merge, broader-scope tests run later.
- Flaky/brittle tests treated as spurious alerts ("If it isn't actually violating the invariants of the SUT, it shouldn't be a test failure"; 100% green argued to be as costly as 100% uptime). → Warns a testing strategy must manage flakiness, relevant given LLM code's non-determinism (cross-ref 2308.02828).
- Hermetic testing as a mitigation ("entirely self-contained ... no external dependencies"), with the Assistant figure that hermetic presubmit "cut the runtime by a factor of 14, with virtually no flakiness." → Argues for hermetic/deterministic test environments to keep an LLM-code suite stable.
- Takeout Scenario 1 operational figures: sandboxed presubmit "prevented 95% of broken servers from bad configuration and reduced nightly deployment failures by 50%." → Illustrates payoff of moving config/startup tests earlier; embedded metric, not a controlled result.

## Evidence weight
Non-empirical context: a 2020 book chapter / experience report whose quantitative figures (95%+ presubmit pass-through, factor-of-14 runtime cut, 95%/50% Takeout numbers) are operational narrative, not outputs of a controlled study with baselines or variance, and it is general software engineering, not specific to LLM-authored code.
