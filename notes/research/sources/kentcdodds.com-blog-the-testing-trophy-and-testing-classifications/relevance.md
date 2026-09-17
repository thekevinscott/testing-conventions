# Relevance — The Testing Trophy and Testing Classifications

**Verdict:** relevant (broad-inclusion) — supplies the "Testing Trophy" model and definitions of unit/integration tiers, an alternative to the test pyramid that frames the granularity vocabulary for an LLM-code strategy, while explicitly disclaiming scientific standing.

## Salient sections

- Trophy layers and tier definitions (mid): "Unit tests are those which test units which either have no dependencies (collaborators) or which have those mocked for the test." and "Integration tests are those which test multiple units integrating with one another." → offers a concrete granularity vocabulary an LLM-code strategy can reference; definitional, author's own framing.
- Non-scientific disclaimer (quoting Tim Bray): "let's not kid ourselves that our software-testing tenets constitute scientific knowledge." plus Dodds: "Any attempt to come to a single definition for all these terms is a futile endeavor." → directly flags that the trophy and these definitions are not empirical, bounding how much weight a strategy should give them.
- Searls quote on the percentage debate (mid): "People love debating what percentage of which type of tests to write, but it's a distraction... Focus on [expressive, fast, reliable tests that only fail for useful reasons] instead." → a quality-over-ratio stance relevant to judging LLM-generated tests; practitioner opinion.
- Scope limitation (self-stated): trophy considered only for monoliths, "not microservices or even serverless functions" → caveats the model's applicability to LLM-authored backend code.

## Evidence weight

Type = practitioner opinion / personal-history essay by Kent C. Dodds (narrative + embedded tweets, no experiments; author disclaims scientific standing); non-empirical context, not primary evidence.
