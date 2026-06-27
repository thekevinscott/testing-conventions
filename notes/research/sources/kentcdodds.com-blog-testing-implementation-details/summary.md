# Summary

- **Title:** Testing Implementation Details
- **Author:** Kent C. Dodds
- **URL:** https://kentcdodds.com/blog/testing-implementation-details
- **Date:** August 17th, 2020 (12 min read)
- **Source type:** (practitioner blog / opinion) — an opinion/tutorial essay. Claims are the
  author's position, not empirical findings; treated as such below.

## What the source claims

Dodds argues that tests which assert on **implementation details** (component internals like
state field names and instance methods, e.g. via enzyme) are harmful, and that tests should
instead interact with software the way its users do (e.g. via React Testing Library). He frames
two failure modes:

> "There are two distinct and important reasons to avoid testing implementation details. Tests
> which test implementation details:
> 1. Can break when you refactor application code. **False negatives**
> 2. May not fail when you break application code. **False positives**"

His definitions, quoted:

> "**This is what's called a false negative.** It means that we got a test failure, but it was
> because of a broken test, not broken app code."

> "**This is called a false positive.** It means that we didn't get a test failure, but we should
> have!"

His definition of implementation details:

> "Implementation details are things which users of your code will not typically use, see, or even
> know about."

On the "two users" framing:

> "**End-users and developers are the two "users" that our application code needs to consider.**"

> "by making our test use the component differently than end-users and developers do, we create a
> third user our application code needs to consider: the tests! And frankly, the tests are one
> user that nobody cares about."

The guiding maxim (self-quoted):

> "[The more your tests resemble the way your software is used, the more confidence they can give
> you.] — me"

> "*Automated tests should verify that the application code works for the production users.*"

He notes enzyme's specific trouble with hooks as additional evidence that implementation-detail
testing impedes refactors: "when you're testing implementation details, a change in the
implementation has a big impact on your tests."

Closing process (how to choose what to test), quoted as a 5-step list:

> "1. What part of your untested codebase would be really bad if it broke? ...
> 2. Try to narrow it down to a unit or a few units of code ...
> 3. Look at that code and consider who the "users" are ...
> 4. Write down a list of instructions for that user to manually test that code ...
> 5. Turn that list of instructions into an automated test."

## Method / evidence type

- **Opinion / tutorial.** Argument by a single worked example: an `Accordion` React class
  component, tested first with enzyme (asserting on `state('openIndex')` and
  `instance().setOpenIndex()`) to demonstrate a false negative under refactor and a false
  positive under a broken click handler, then re-tested with React Testing Library to show both
  problems disappear.
- Evidence is illustrative code and the author's reasoning/experience; no studies, measurements,
  or data.

## Numbers recorded

**None.** No quantitative results. (The "19,410" and "12 min read" are site read-count and
reading-time metadata.) The author at one point sarcastically references adding "a coverage
threshold of 100% code coverage," but this is rhetorical, not a measured figure.

## Scope, limitations, and gaps

- **Single-author opinion**, scoped to frontend/React with enzyme vs. React Testing Library
  (a library the author created — a relevant conflict of interest, disclosed in the text).
- Examples use a dated class component; the author acknowledges this and ties it to enzyme's
  hooks limitations rather than presenting a controlled comparison.
- "False positive/false negative" terminology is the author's own framing (he redefines the
  usual statistical sense); he spells this out but it diverges from conventional usage.
- No empirical claim about how often implementation-detail tests cause real-world defects; the
  "Nancy can't get her tickets" production-failure story is a hypothetical illustration.

## Capture status

`transcript.md` is the full article (curl, `capture_status: ok`): all prose, the `accordion.js`
source, the enzyme test, the refactor diff, the corrected enzyme test, and the React Testing
Library test, plus the closing process list. CodeSandbox link and course pages are referenced but
not captured.
