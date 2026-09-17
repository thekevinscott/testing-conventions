# Summary

- **Title:** Stop mocking fetch
- **Author:** Kent C. Dodds
- **URL:** https://kentcdodds.com/blog/stop-mocking-fetch
- **Date:** June 3rd, 2020 (11 min read)
- **Source type:** (practitioner blog / opinion) — an opinion/tutorial essay. Claims are the
  author's position and personal experience, not empirical findings; treated as such below.

## What the source claims

Dodds argues against mocking the HTTP layer (the `client` wrapper or `window.fetch`) in
integration tests, because doing so forces you to re-implement your backend across many tests and
loses confidence that requests are actually formed correctly. His recommended alternative is
**`msw` (Mock Service Worker)** — intercepting requests at a mock server defined once and reused
across tests and development.

Verbatim quotes:

On the cost of mocking the client / fetch:

> "One thing that really bothers me about mocking things like `fetch` is that you end up
> re-implementing your entire backend... everywhere in your tests. Often in multiple tests."

> "Ultimately, we have less confidence, a slower feedback loop, lots of duplicate code, or any
> combination of those."

On the confidence gap when mocking the client object:

> "because you're mocking out the `client`, how do you really know the client is being used
> correctly in this case?"

On why he prefers `msw`:

> "1. I don't have to worry about the implementation details of fetch response properties and
> headers.
> 2. If I get something wrong with the way I call `fetch`, then my server handler won't be called
> and my test (correctly) fails, which would save me from shipping broken code.
> 3. I can reuse these exact same server handlers in my development!"

On colocation of edge/error cases:

> "So you can have colocation where it's needed, and abstraction where abstraction is sensible."

Concluding principle (the refactoring-confidence payoff):

> "because you're so far away from implementation details, you can make significant refactorings
> and your tests can give you confidence that you didn't break the user experience. That's what
> tests are for!!"

He also describes a pre-`msw` intermediate technique he used ("a form of this at PayPal and it
worked really well"): a single `mockFetch` function that re-implements the tested parts of the
backend via a `switch` on URL.

## Method / evidence type

- **Opinion / tutorial.** Argument by progressively refactored code examples for one React
  `Checkout` test: (1) mocking the `client`, (2) mocking `window.fetch`, (3) a hand-rolled
  `mockFetch` backend, (4) `msw` server handlers shared between tests and development.
- Evidence is the author's stated experience plus two embedded supporting tweets (from Dodds and
  from a developer "@d11erh"). No measurements, benchmarks, or studies.
- Compares `msw` to alternatives `nock` and `Mirage`, noting `msw`'s use of a service worker so
  "the network tab works the same."

## Numbers recorded

**None.** No quantitative data — no measured confidence, speed, defect, or coverage figures. The
"13,816" and "11 min read" on the page are site read-count and reading-time metadata, not study
results.

## Scope, limitations, and gaps

- **Single-author opinion**, JavaScript/React-specific (`@testing-library/react`, Jest, `msw`).
- Recommendation is tool-specific (`msw`); the author has commercial testing courses
  (TestingJavaScript.com, EpicReact.dev) linked throughout — a potential conflict of interest to
  note.
- The author concedes a real downside ("One reasonable concern about this approach is that you
  end up putting all of your server handlers in one place and then the tests ... end up in
  entirely different files, so you lose the benefits of colocation") and answers it with runtime
  handler overrides — a mitigation, not a measured trade-off.
- No discussion of when mocking at the fetch boundary is preferable, nor of `msw` setup cost
  beyond "fast and easy to write (once you have things set up)."

## Capture status

`transcript.md` is the full article (curl, `capture_status: ok`): all prose, every code block
(client mock, fetch mock, `mockFetch`, the `msw` `server-handlers.js` / `server.js` /
`setup-env.js` / `checkout.js` examples), and the two embedded tweets. Linked workshop/GitHub
material and the course pages are referenced but not captured.
