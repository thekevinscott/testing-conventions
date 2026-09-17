# Summary

- **Title:** Test Double
- **Author / org:** Martin Fowler (martinfowler.com / Thoughtworks)
- **URL:** https://martinfowler.com/bliki/TestDouble.html
- **Date:** 17 January 2006
- **Venue:** Personal site (martinfowler.com) "bliki", "testing" tag
- **Source type:** (practitioner article / definitional essay) by Martin Fowler — opinion/definitions, not a study

## What the source claims

A short glossary entry that popularizes Gerard Meszaros's vocabulary for the objects that
stand in for real ones in tests. "Test Double" is offered as the generic umbrella term, with
five named kinds. Purely definitional; no argument or evidence.

Verbatim quotes:

> "Test Double is a generic term for any case where you replace a production object for
> testing purposes."

> "**Dummy** objects are passed around but never actually used. Usually they are just used
> to fill parameter lists."

> "**Fake** objects actually have working implementations, but usually take some shortcut
> which makes them not suitable for production (an InMemoryTestDatabase is a good example)."

> "**Stubs** provide canned answers to calls made during the test, usually not responding at
> all to anything outside what's programmed in for the test."

> "**Spies** are stubs that also record some information based on how they were called. One
> form of this might be an email service that records how many messages it was sent."

> "**Mocks** are pre-programmed with expectations which form a specification of the calls
> they are expected to receive. They can throw an exception if they receive a call they
> don't expect and are checked during verification to ensure they got all the calls they
> were expecting."

## Method / evidence type

Opinion / definitional glossary entry. Attributes the taxonomy to Gerard Meszaros's
forthcoming xUnit patterns book. No method, data, or experiment.

## Numbers recorded

None. The only enumeration is the **five** kinds of test double (Dummy, Fake, Stub, Spy, Mock).

## Scope, limitations, and gaps

- Definitions only; the "when/why" discussion is deferred to the companion essay
  ("Mocks Aren't Stubs").
- Vocabulary is one author's (Meszaros) convention, not a universal standard.

## Capture status

`transcript.md` is the **full bliki entry** (curl, `ok`): the framing paragraph, the
five-item definition list, and the "Further Reading" pointer to "Mocks Aren't Stubs."
Remainder is site chrome/nav.
