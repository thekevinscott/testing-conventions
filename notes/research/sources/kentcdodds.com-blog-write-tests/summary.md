# Summary

- **Title:** Write tests. Not too many. Mostly integration.
- **Author:** Kent C. Dodds (expanding on a Guillermo Rauch tweet)
- **URL:** https://kentcdodds.com/blog/write-tests
- **Date:** July 13th, 2019 (6 min read)
- **Source type:** (practitioner blog / opinion) — an opinion essay unpacking a slogan. Claims
  are the author's position, not empirical findings.

## What the source claims

Dodds expands Guillermo Rauch's slogan **"Write tests. Not too many. Mostly integration."** into
three arguments: (1) write automated tests for confidence; (2) do not chase 100% coverage —
returns diminish; (3) favor integration tests for the best confidence-per-effort, largely by
mocking less.

Verbatim quotes:

On "Write tests":

> "for most projects you should write automated tests. You should if you value your time anyway.
> Much better to catch a bug locally from the tests than getting a call at 2:00 in the morning"

> "The thing you should be thinking about when writing tests is how much confidence they bring you
> that your project is free of bugs."

> "even a strongly typed language should have tests. Typing and linting can't ensure your business
> logic is free of bugs."

On "Not too many" (the made-up 70% figure, disclosed as such):

> "I've heard managers and teams mandating 100% code coverage for applications. That's a really
> bad idea. The problem is that **you get diminishing returns on your tests as the coverage
> increases much beyond 70%** (I made that number up... no science there)."

> "You should very rarely have to change tests when you refactor code."

He notes his own open-source libraries are an exception ("almost all of my open source projects
have 100% code coverage"), justified by their reusability and small size.

On "Mostly integration" and the pyramid → confidence point:

> "as you move up the pyramid, the confidence quotient of each form of testing increases. You get
> more bang for your buck."

> "**Integration tests strike a great balance on the trade-offs between confidence and
> speed/expense.** This is why it's advisable to spend *most* (not all, mind you) of your effort
> there."

On mocking (the mechanism for writing more integration tests):

> "the biggest thing you can do to write more integration tests is to **stop mocking so much
> stuff**. *When you mock something you're removing all confidence in the integration between what
> you're testing and what's being mocked.*"

> "**If you're doing React, then this includes shallow rendering.**"

## Method / evidence type

- **Opinion / commentary.** Builds on Rauch's tweet; references a testing pyramid slide (credited
  to Martin Fowler's blog and the Google Testing blog) and embedded tweets. The post is also
  available as a recorded talk (linked, not captured).
- No experiments, datasets, or measurements; reasoning is from the author's experience.

## Numbers recorded

**No study numbers.** The only figure, "70%" (diminishing-returns coverage threshold), is
explicitly **invented**: "I made that number up... no science there." "100% code coverage" is
cited as a (bad) management mandate and as a property of his own OSS projects, not as evidence.
("33,426" and "6 min read" are site metadata.)

## Scope, limitations, and gaps

- **Single-author opinion**, frontend/React-leaning (shallow rendering, components).
- The headline coverage threshold is self-admittedly unscientific; the author offers no data for
  the diminishing-returns claim or the confidence-per-effort ranking of test types.
- Acknowledges mocking "sometimes it can't be helped" (don't really send emails/charge cards) and
  links opposing views — a stated nuance rather than a measured boundary.
- Promotes the author's paid courses/workshops; advocacy and instruction are intertwined.

## Capture status

`transcript.md` is the full article (curl, `capture_status: ok`): all prose (Write tests / Not
too many / Mostly integration / How to write more integration tests / Conclusion), and embedded
tweets (Rauch, Dodds). The testing-pyramid image, the linked talk video, and slide deck are
referenced but not captured.
