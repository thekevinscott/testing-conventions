# Relevance — Mocks Aren't Stubs

**Verdict:** relevant (broad-inclusion) — the canonical classicist-vs-mockist essay; defines the test-double and state-vs-behavior-verification vocabulary needed to even discuss the mocking dimension of an LLM-code testing strategy, though it supplies no measurement.

## Salient sections

- State vs behavior verification, and classical vs mockist TDD (opening): "a difference in how test results are verified: a distinction between state verification and behavior verification... the classical and mockist styles of Test Driven Development." → defines the axes along which an LLM-code strategy decides how to use doubles; definitional.
- Mockist coupling claim (mid): "Mockist tests are thus more coupled to the implementation of a method. Changing the nature of calls to collaborators usually cause a mockist test to break." → bears on whether mock-heavy tests over LLM-authored code are brittle; Fowler's reasoned opinion, not data.
- Classic tests as mini-integration (mid): "In essence classic xunit tests are not just unit tests, but also mini-integration tests." → connects double-choice to test granularity, relevant to the goal's granularity dimension.
- Fowler's own position (near end): "I've always been a old fashioned classic TDDer... I don't see any compelling benefits for mockist TDD, and am concerned about the consequences of coupling tests to implementation." plus his caveat that he has not tried mockist TDD "on anything more than toys" → an attributed preference with a self-stated limitation, underscoring this is opinion.

## Evidence weight

Type = practitioner / definitional essay by Martin Fowler (Java examples + experience, no measurement or dataset); non-empirical context, not primary evidence.
