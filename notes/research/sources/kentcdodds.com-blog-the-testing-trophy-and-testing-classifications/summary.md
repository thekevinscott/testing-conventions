# Summary

- **Title:** The Testing Trophy and Testing Classifications
- **Author:** Kent C. Dodds
- **URL:** https://kentcdodds.com/blog/the-testing-trophy-and-testing-classifications
- **Date:** June 3rd, 2021 (7 min read)
- **Source type:** (practitioner blog / opinion) — an opinion/history essay defining the author's
  own testing terms. Claims are the author's position, not empirical findings.

## What the source claims

Dodds gives the origin and intended interpretation of the **"Testing Trophy"** (four layers: End
to End, Integration, Unit, Static) and explains his **own loose definitions** of unit vs.
integration tests. He stresses the classifications are inherently contested and ultimately
secondary to writing good tests.

His definitions, quoted:

> "Unit tests are those which test units which either have no dependencies (collaborators) or
> which have those mocked for the test."

> "Integration tests are those which test multiple units integrating with one another."

> "I consider a "unit" to be a single function, class, or object that contains logic."

On where the trophy applies (a scope limitation he states himself):

> "I never considered whether it applied to microservices or even backend services at all. I
> considered my codebase in isolation ..."

> "It definitely has applicability in backends, but I've only considered it for monoliths not
> microservices or even serverless functions."

On why he invented his own definitions, and the field's lack of consensus — he cites Martin
Fowler approximating a "test expert":

> "in the first morning of my training course I cover 24 different definitions of unit test"

He quotes Tim Bray on the non-scientific status of testing beliefs:

> "let's not kid ourselves that our software-testing tenets constitute scientific knowledge."

> "I would say this applies to everything about testing–not just whether it's effective (it can
> be). Any attempt to come to a single definition for all these terms is a futile endeavor."

He endorses Justin Searls' view that the percentage debate is a distraction:

> "People love debating what percentage of which type of tests to write, but it's a distraction.
> Nearly zero teams write expressive tests that establish clear boundaries, run quickly &
> reliably, and only fail for useful reasons. Focus on that instead."

The guiding principle (self-quoted) and ROI framing:

> "The more your tests resemble the way your software is used, the more confidence they can give
> you."

> "it's all about getting a good return on your investment where "return" is "confidence" and
> "investment" is "time.""

Lineage noted: the trophy descends from Guillermo Rauch's 2016 tweet "Write tests. Not too many.
Mostly integration." and was introduced by Dodds in a February 2018 tweet; "static" was added
because in JavaScript it is "not a given like it is in the predominant languages when the testing
pyramid was introduced."

## Method / evidence type

- **Opinion / personal history.** Narrative tracing the trophy's origin, supported by embedded
  tweets (Dodds, Rauch, Searls, swyx) and links to Martin Fowler and Tim Bray articles.
- No experiments, data, or measurements. The author explicitly disclaims scientific standing for
  testing tenets.

## Numbers recorded

**No study numbers.** The only figures are anecdotal/third-party: Fowler's quoted "24 different
definitions of unit test," and tweet engagement counts. The author offers no measured comparison
of test types. (Note: in the linked "Write tests" post he separately calls a "70%" coverage
figure one he "made up... no science there.")

## Scope, limitations, and gaps

- **Single-author opinion**, frontend/JavaScript-centric ("almost all the code I wrote either
  ran directly in a browser"); applicability to microservices/serverless explicitly excluded.
- Definitions are admittedly idiosyncratic ("I had to choose something that made sense for me");
  the author pre-empts the objection that he should have used existing definitions.
- The author markets Testing Library (which he created) and paid courses; the post doubles as
  advocacy for those tools.
- Effectiveness of the trophy is asserted from teaching response ("Judging by the response from
  people who have implemented my recommendations, my decision was a good one"), not measured.

## Capture status

`transcript.md` is the full article (curl, `capture_status: ok`): all prose, the embedded tweets
(Dodds, Rauch, Searls, swyx), and the closing recommended-reading list. Trophy/pyramid images and
linked articles are referenced but not captured.
