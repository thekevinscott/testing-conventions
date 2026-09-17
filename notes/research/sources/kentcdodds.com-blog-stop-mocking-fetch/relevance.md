# Relevance — Stop mocking fetch

**Verdict:** relevant (broad-inclusion) — practitioner opinion on where to place the mock boundary; informs the test-doubles/mocking axis of the strategy, as context rather than evidence.

## Salient sections
- The thesis (`summary.md` lines 13–17, Dodds's position): don't mock the HTTP layer (`client`/`window.fetch`) in integration tests; intercept at a mock server defined once via **`msw` (Mock Service Worker)**. Speaks to the strategy's question of which boundary to stub.
- Cost of mocking fetch (`summary.md` lines 22–27): "you end up re-implementing your entire backend... everywhere in your tests," yielding "less confidence, a slower feedback loop, lots of duplicate code." A maintainability/confidence argument against low-level mocking.
- Confidence gap (`summary.md` lines 29–38): "because you're mocking out the `client`, how do you really know the client is being used correctly?"; with msw, "If I get something wrong with the way I call `fetch`, then my server handler won't be called and my test (correctly) fails." Relevant to what makes a test trustworthy.
- Refactoring payoff (`summary.md` lines 44–49): testing "far away from implementation details" lets you "make significant refactorings and your tests can give you confidence that you didn't break the user experience." This is the author's claimed benefit, not a measured one.
- Caveats (`summary.md` lines 72–82): single-author, JS/React-specific (`msw`/Jest/testing-library); summary flags a potential conflict of interest (author sells testing courses). No benchmarks or studies.

## Evidence weight
Practitioner blog / opinion (single author, one React `Checkout` example plus two tweets); non-empirical — no measurements, and not about LLM-authored code.
