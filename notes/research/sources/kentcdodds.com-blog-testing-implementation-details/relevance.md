# Relevance — Testing Implementation Details

**Verdict:** relevant (broad-inclusion) — supplies practitioner vocabulary and an argument for *what to assert* (user-visible behavior, not internals) that frames how one might judge tests over LLM-authored code, though it offers no measurement.

## Salient sections

- Two-failure-mode framing (opening): "Tests which test implementation details: 1. Can break when you refactor application code. **False negatives** 2. May not fail when you break application code. **False positives**" → for LLM-authored code, argues a strategy should prefer assertions that survive refactors and still catch real breakage; relevant as a stated goal, not a measured result.
- Definition of implementation details (mid): "Implementation details are things which users of your code will not typically use, see, or even know about." → suggests the gate for an LLM-code test is observable behavior rather than internal structure.
- Guiding maxim (self-quoted): "The more your tests resemble the way your software is used, the more confidence they can give you. — me" → an opinion heuristic the LLM-code strategy could adopt, attributable to Dodds, not evidence.
- Closing 5-step process (end): pick what would be bad if broken → narrow to units → identify the "users" → write manual instructions → automate them → a procedure for deriving tests, usable when a human reviews LLM-generated tests.

## Evidence weight

Type = practitioner opinion / tutorial essay by Kent C. Dodds (single React example, no studies or data); non-empirical context, not primary evidence.
