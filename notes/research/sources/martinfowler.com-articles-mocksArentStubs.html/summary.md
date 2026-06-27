# Summary

- **Title:** Mocks Aren't Stubs
- **Author / org:** Martin Fowler (martinfowler.com / Thoughtworks)
- **URL:** https://martinfowler.com/articles/mocksArentStubs.html
- **Date:** 02 January 2007 (first published 08 July 2004; tagged "Significant Revisions")
- **Venue:** Personal site (martinfowler.com), "testing" tag
- **Source type:** (practitioner article / definitional essay) by Martin Fowler — opinion/definitions, not a study

## What the source claims

Fowler distinguishes two separate things people conflate: (1) **state verification vs.
behavior verification** (how a test checks results), and (2) **classical vs. mockist TDD**
(when to use a double). He defines the five Meszaros "test doubles" and argues mocks are
only one kind. The piece is opinion and pedagogy, not empirical evidence; Fowler states his
own preference (classical) while presenting both sides.

Verbatim quotes:

> "This difference is actually two separate differences. On the one hand there is a
> difference in how test results are verified: a distinction between state verification and
> behavior verification. On the other hand is a whole different philosophy to the way
> testing and design play together, which I term here as the classical and mockist styles
> of Test Driven Development."

> "Meszaros uses the term **Test Double** as the generic term for any kind of pretend object
> used in place of a real object for testing purposes."

> "Of these kinds of doubles, only mocks insist upon behavior verification. The other
> doubles can, and usually do, use state verification."

> "The **classical TDD** style is to use real objects if possible and a double if it's
> awkward to use the real thing... A **mockist TDD** practitioner, however, will always use
> a mock for any object with interesting behavior."

> "Mockist tests are thus more coupled to the implementation of a method. Changing the
> nature of calls to collaborators usually cause a mockist test to break."

> "In essence classic xunit tests are not just unit tests, but also mini-integration tests."

On his own position:

> "Personally I've always been a old fashioned classic TDDer and thus far I don't see any
> reason to change. I don't see any compelling benefits for mockist TDD, and am concerned
> about the consequences of coupling tests to implementation."

> "whichever style of test you use, you must combine it with coarser grained acceptance
> tests that operate across the system as a whole."

## Method / evidence type

Opinion / definitional essay. Evidence is worked Java code examples (Order/Warehouse with
jMock and EasyMock; a MailService stub) and the author's experience and conversations with
mock-tool developers. No measurement, dataset, or experiment.

## Numbers recorded

None. The essay reports no quantitative results. The only enumerations are categorical:
- **Five** kinds of test double (Dummy, Fake, Stub, Spy, Mock).
- xUnit's **four**-phase test sequence (setup, exercise, verify, teardown).
- A rule-of-thumb that test clusters should span "no more than half a dozen" objects.

## Scope, limitations, and gaps

- Explicitly opinion; Fowler notes he has not "[tried] mockist TDD on anything more than
  toys," limiting the weight of his preference.
- Vocabulary is Meszaros's (xUnit Test Patterns); Fowler notes "It's not what everyone
  uses." Terms like "Detroit"/"London" for classical/mockist are noted as disliked by some.
- Definitional, not prescriptive about a project's overall test mix (that is the separate
  Test Pyramid bliki).

## Capture status

`transcript.md` is the **full article** (curl, `ok`): all sections (Regular Tests; Tests
with Mock Objects incl. EasyMock; The Difference Between Mocks and Stubs; Classical and
Mockist Testing; Choosing Between the Differences with its sub-sections; So should I be a
classicist or a mockist?; Final Thoughts; Further Reading; Significant Revisions). Code
listings are included inline. Only site chrome/nav is extraneous.
