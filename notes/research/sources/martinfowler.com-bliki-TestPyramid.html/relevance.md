# Relevance — Test Pyramid

**Verdict:** relevant (broad-inclusion) — the canonical test-pyramid heuristic for test-portfolio shape, directly framing the granularity dimension of the goal (what to test at which level), though it is explicitly a heuristic, not a measured optimum.

## Salient sections

- Core heuristic (opening): "you should have many more low-level UnitTests than high level BroadStackTests running through a GUI." → a default granularity distribution a strategy for LLM-authored code could adopt; Fowler's advice, not a measured result.
- End-to-end cost claim (mid): "tests that run end-to-end through the UI are: brittle, expensive to write, and time consuming to run." → rationale for weighting toward lower-level tests; asserted from experience.
- High-level tests as second line of defense (mid): "before fixing a bug exposed by a high level test, you should replicate the bug with a unit test. Then the unit test ensures the bug stays dead." → a procedure relevant to layering tests over LLM-generated code.
- Self-stated exception (caveat): "If my high level tests are fast, reliable, and cheap to modify - then lower-level tests aren't needed." and the note that pyramid-vs-more-integration disagreement is "probably illusory due to different definitions of 'unit test'" → bounds the heuristic and flags that the shape debate is largely terminological, not empirical.

## Evidence weight

Type = practitioner / definitional heuristic essay by Martin Fowler (experience-based, presents no data of its own); non-empirical context, not primary evidence.
