# Summary

- **Title:** "Don't Mock What You Don't Own" in 5 Minutes
- **Author:** Hynek Schlawack
- **URL:** https://hynek.me/articles/what-to-mock-in-5-mins/
- **Date:** 21 June 2022
- **Source type:** (practitioner blog / opinion) — an opinion essay restating a testing
  heuristic, with a worked Python example. Claims below are the author's position, not empirical
  findings.

## What the source claims

Schlawack argues for the **"Don't Mock What You Don't Own"** principle: mock objects should
substitute *your own* objects, not third-party ones. His core reframing is that ownership of the
*API to use an object* matters more than ownership of the *object* itself, and the fix is to wrap
third-party dependencies behind a thin façade you own, then mock that façade.

Verbatim quotes:

> "*Don't Mock What You Don't Own* means that whenever you employ mock objects, you should use
> them to substitute your ***own*** objects and not third-party ones."

> "**Key point:** there's a difference between owning an **object** and owning the **API** to use
> it."

On why mocking third-party objects directly is bad (the "rude mocking" example needing three
nested mocks):

> "We need **three** layers of mocks to verify that an empty `repositories` key leads to an empty
> dictionary. And if I didn't use a `lambda` for the `json` function, it would be even four
> layers."

> "This is a *business logic* test and the purpose of the test is drowning in boilerplate
> necessary to mimic the API of an HTTP client that can change at any time."

> "That makes the test **brittle** and **unidiomatic**. When I read tests for business logic, I
> want *the intent* of the test to be obvious on first glimpse."

> "Eventually, you end up in *Mock Hell*."

On the payoff of the façade ("polite mocking"):

> "The first pay-off is that the business logic is more idiomatic!"

> "**This point can't be overstated**: In an attempt to simplify our mocks by following a
> counter-intuitive principle from ancient times, we've *improved our business logic*."

> "Only one `Mock`! And *one* look and you know what's happening!"

> "And if you choose to replace your HTTP library, your business tests won't care, because they
> only interface with *your* abstraction."

His stated corollary:

> "**Corollary**: To keep your *business code* testable and idiomatic, avoid directly using
> third-party dependencies in it."

The author also discloses he dislikes mocks generally and prefers other test doubles:

> "However, **I** don't use mocks in my *own* code at all. In Python I use *pretend* for simple
> stubs and *verified fakes* for more complex scenarios. In Go I reach always for *verified
> fakes*."

On treating the rule as a heuristic, not a law:

> "Every rule and principle can be broken once you've fully understood its purpose."

> "**In the end, it's less of a rule and more of a heuristic.**"

> "The most common occasion when **I** break this principle is when I need to simulate errors that
> aren't trivial to create organically: certain network conditions, timeouts, integrity
> errors, …"

## Method / evidence type

- **Opinion / tutorial.** Argument by a single worked Python example: a Docker-registry client
  function tested two ways — (1) mocking the third-party `httpx` client directly ("rude"), then
  (2) wrapping it in a `DockerRegistryClient` façade and mocking that ("polite").
- No experiments, measurements, datasets, or surveys. Persuasion is by code comparison and the
  author's experience.
- The principle is attributed to the **London School of TDD**; the post cites Martin Fowler's
  *Mocks Aren't Stubs*, the *Architecture Patterns with Python* book, and other essays as
  further reading.

## Numbers recorded

**None of an empirical kind.** The only quantities are illustrative counts from the example:
"three" (sometimes "four") layers of mocks needed in the naive approach versus "Only one `Mock`"
after introducing the façade. These are properties of the toy example, not measured results.

## Scope, limitations, and gaps

- **Single author opinion**, explicitly framed as a 5-minute talk written up; no claim to
  generality or evidence.
- Example is deliberately small ("To keep my example short") and Python/`httpx`-specific, though
  the author asserts the problem is "universal to any object-oriented language."
- The author himself flags the trade-off: for "simple programs like in this blog post, it's
  probably easier to write a test helper that creates appropriate fake HTTP clients," and adds
  "Trade-offs, trade-offs."
- He concedes the façade can just "kick the proverbial testing can one layer down" if the wrapper
  isn't kept cyclomatically simple, and that the thin outer layer is "notoriously difficult to
  test."
- No discussion of cost/benefit at scale, team adoption, or when the wrapping overhead outweighs
  the benefit beyond the brief "When to break the rule?" section.

## Capture status

`transcript.md` is the full article (curl, `capture_status: ok`): complete prose, all code
listings (the naive function and its 3-layer mock test; the `DockerRegistryClient` façade and its
1-mock test), the three footnotes, and "Further reading" links. Embedded video and the author's
linked talks/tweets are referenced but not captured.
