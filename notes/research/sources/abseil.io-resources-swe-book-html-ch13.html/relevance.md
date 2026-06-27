# Relevance — SWE at Google ch.13: Test Doubles

**Verdict:** relevant (broad-inclusion) — supplies a practitioner preference-ordering for test doubles (real implementations > fakes > stubbing/interaction testing) that a testing strategy for LLM code can adopt for what to mock vs. exercise for real.

## Salient sections
- The chapter's central preference (What-the-source-claims section): "our first choice for tests is to use the real implementations of the system under test's dependencies," preferred when a dependency "is fast, deterministic, and has simple dependencies." → For LLM-authored code, this argues a testing strategy should default to real-dependency (classical) tests and reserve doubles for narrow cases, rather than mock-heavy tests.
- The reported reversal on mocking (quote in summary): Google found mock-heavy tests "required constant effort to maintain while rarely finding bugs," and the "pendulum ... has now begun swinging in the other direction." → Cautions against over-mocking in an LLM-code test suite because such tests find few bugs and raise maintenance cost.
- Interaction testing guidance ("prefer state testing over interaction testing"; overuse yields "change-detector tests" that "fail in response to any change to the production code, even if the behavior ... remains unchanged"). → Favors state/behavior assertions over call-verification when testing LLM output, to avoid brittle tests.
- Fakes "must have its own tests" via contract tests run against both real and fake. → If LLM code uses fakes, fidelity must itself be tested.

## Evidence weight
Non-empirical context: a 2020 book chapter of experience-based engineering guidance (no dataset, effect sizes, or measured bug-finding/maintenance outcomes), and general software engineering, not specific to LLM-authored code — context for the strategy, not primary evidence for an "optimal" claim.
