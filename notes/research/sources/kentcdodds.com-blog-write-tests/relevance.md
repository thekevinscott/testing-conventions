# Relevance — Write tests. Not too many. Mostly integration.

**Verdict:** relevant (broad-inclusion) — articulates a widely-cited practitioner stance on coverage targets, the integration-tier emphasis, and mocking-less, all of which touch the strategy dimensions in the goal; but its headline figure is self-admittedly invented.

## Salient sections

- The made-up 70% coverage figure (under "Not too many"): "you get diminishing returns on your tests as the coverage increases much beyond 70% (I made that number up... no science there)." → bears on the goal's coverage-target dimension only as an opinion the author explicitly flags as unscientific; must not be cited as an empirical threshold.
- Confidence-per-effort ranking (under "Mostly integration"): "Integration tests strike a great balance on the trade-offs between confidence and speed/expense. This is why it's advisable to spend most (not all) of your effort there." → a granularity recommendation relevant to an LLM-code strategy; asserted from experience, not measured.
- Mocking guidance (under "How to write more integration tests"): "stop mocking so much stuff. When you mock something you're removing all confidence in the integration between what you're testing and what's being mocked." → speaks to the test-double dimension of the goal; practitioner opinion.
- OSS exception (mid): "almost all of my open source projects have 100% code coverage," justified by reusability/small size → directly touches open-source code (a goal term), as an anecdotal practice, not evidence.

## Evidence weight

Type = practitioner opinion essay by Kent C. Dodds (slogan commentary, no datasets; the 70% figure is openly invented); non-empirical context, not primary evidence.
