# Relevance — "Don't Mock What You Don't Own" in 5 Minutes

**Verdict:** relevant (broad-inclusion) — practitioner opinion on the mocking/test-doubles axis of the testing-strategy definition; supplies a named heuristic and its rationale, not empirical evidence.

## Salient sections
- Core heuristic (`summary.md` lines 19–25, Schlawack's words): "whenever you employ mock objects, you should use them to substitute your ***own*** objects and not third-party ones," with the reframing that owning the **API** to use an object matters more than owning the object. Directly addresses the strategy's "test-doubles/mocking" sub-question.
- "Mock Hell" cost (`summary.md` lines 31–39): mocking a third-party HTTP client directly needs "**three**" (sometimes four) nested mocks, making the test "**brittle** and **unidiomatic**." Argues brittle over-mocking as a failure mode to avoid.
- Façade payoff (`summary.md` lines 43–52): wrapping the dependency yields "Only one `Mock`!" and tests that "won't care" if the HTTP library is swapped — a maintainability argument for the wrap-and-mock-your-own pattern.
- Heuristic-not-law framing (`summary.md` lines 64–73): "it's less of a rule and more of a heuristic"; the author breaks it to simulate hard-to-create errors (timeouts, network conditions). Author also discloses he avoids mocks entirely, preferring fakes (lines 58–62). Attribution: this is Schlawack's stated position (London-School-of-TDD lineage), explicitly not measured.

## Evidence weight
Practitioner blog / opinion (single author, one toy Python `httpx` example); non-empirical — no experiments, datasets, or measured results, and not specific to LLM-authored code.
