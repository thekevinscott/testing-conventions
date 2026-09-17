# Summary

- **Title:** Test Doubles (Chapter 13 of *Software Engineering at Google*)
- **Authors:** Andrew Trenk and Dillon Bly (edited by Tom Manshreck)
- **URL:** https://abseil.io/resources/swe-book/html/ch13.html
- **Date:** Not dated on page (book published 2020); fetched 2026-06-27
- **Venue:** *Software Engineering at Google* (the "SWE Book"), O'Reilly / Google
- **Source type:** (book chapter)

## What the source claims

The chapter is Google's practitioner guidance on *test doubles* — objects or functions
that stand in for real implementations in tests. It defines the central concepts, the three
double techniques, and a strong preference ordering: real implementations first, then fakes,
with stubbing and interaction testing constrained to narrow cases.

A test double is defined by analogy:

> "A *test double* is an object or function that can stand in for a real implementation in a
> test, similar to how a stunt double can stand in for an actor in a movie."

Three trade-off dimensions are introduced for using doubles: **testability** (the codebase
must be designed so real implementations can be swapped out — typically via *seams* and
*dependency injection*), **applicability** (improper use "can lead to tests that are brittle,
complex, and less effective"), and **fidelity** ("how closely the behavior of a test double
resembles the behavior of the real implementation").

A core narrative claim is that Google over-used mocking frameworks and reversed course:

> "When mocking frameworks first came into use at Google, they seemed like a hammer fit for
> every nail... It wasn't until several years and countless tests later that we began to
> realize the cost of such tests: though these tests were easy to write, we suffered greatly
> given that they required constant effort to maintain while rarely finding bugs. The
> pendulum at Google has now begun swinging in the other direction, with many engineers
> avoiding mocking frameworks in favor of writing more realistic tests."

**Three techniques** for using doubles are defined:
- *Faking* — "a lightweight implementation of an API that behaves similar to the real
  implementation but isn't suitable for production; for example, an in-memory database."
- *Stubbing* — "the process of giving behavior to a function that otherwise has no behavior
  on its own — you specify to the function exactly what values to return."
- *Interaction testing* — "a way to validate *how* a function is called without actually
  calling the implementation of the function."

**Preference for real implementations** (called *classical testing*, contrasted with
*mockist testing*):

> "our first choice for tests is to use the real implementations of the system under test's
> dependencies"

> "at Google, we have found that this [mockist] style of testing is difficult to scale."

A real implementation is preferred "if it is fast, deterministic, and has simple
dependencies." When not feasible, the decision turns on **execution time**, **determinism**
(non-hermetic code and reliance on the system clock cause flakiness), and **dependency
construction** cost.

The `@DoNotMock` annotation (in Google's ErrorProne static-analysis tool) lets API owners
declare "this type should not be mocked because better alternatives exist." Rationale: heavy
mocking of a type across the codebase freezes the API owner's ability to change the
implementation, because thousands of test doubles encode behavior that "violates the API
contract of the type being mocked."

**Fakes** are presented as the best fallback when real implementations can't be used; they
should be written/maintained by the team that owns the real implementation, must maintain
fidelity to the real implementation's **API contract**, and "must have its *own* tests"
(via *contract tests* run against both the real implementation and the fake).

**Stubbing** dangers: overuse makes tests unclear, brittle ("Stubbing leaks implementation
details of your code into your test"), and less effective (the stub can't be guaranteed to
behave like the real implementation, and can't store state). Stubbing is appropriate only
when "you need a function to return a specific value to get the system under test into a
certain state," and a test "should stub out a small number of functions."

**Interaction testing**: prefer *state testing* over interaction testing. Overuse produces
*change-detector tests* "because they fail in response to any change to the production code,
even if the behavior of the system under test remains unchanged." Best practices when it is
used: perform it "only for functions that are state-changing" (e.g. `sendEmail()`,
`saveRecord()`, not `getUser()`, `readFile()`), and "avoid overspecifying which functions and
arguments are validated."

## Method / evidence type

Experience-based engineering guidance from Google. Evidence is illustrative Java/Mockito code
examples and reported organizational experience ("we've seen countless examples", "we
suffered greatly"). No quantitative study, dataset, or measured results — claims are
qualitative and drawn from internal practice.

## Numbers recorded

The chapter contains essentially no quantitative measurements. The only numeric/illustrative
references in the text:
- A rhetorical latency discussion: "what if it added 10 milliseconds, 100 milliseconds, 1
  second" per test case; "one second extra per test case may be reasonable if there are five
  test cases, but not if there are 500."
- A fake "might not need to have 100% of the functionality of its corresponding real
  implementation."
- Object construction example: `Foo foo = new Foo(new A(new B(new C()), new D()), new E(),
  ..., new Z());`

No tables, benchmarks, or measured outcomes are reported.

## Scope, limitations, and gaps

- Guidance is grounded in Google's monorepo, tooling (Guice, Dagger, Bazel, ErrorProne,
  Mockito, googlemock, unittest.mock) and culture; the chapter itself notes "the actual
  application of them varies widely from team to team."
- Claims are experiential, not empirically measured — no effect sizes for bug-finding,
  maintenance cost, or flakiness reduction are provided.
- Examples are Java-centric (with brief notes on dynamically typed languages like Python and
  JavaScript where dependency injection is "less important").
- The chapter explicitly defers larger-scope testing (exercising real dependencies that are
  slow or nondeterministic) to the next chapter.

## Capture status

`transcript.md` (curl, `capture_status: ok`) contains the **full chapter text**: all prose
sections, code Examples 13-1 through 13-19, the `@DoNotMock` and fidelity case-study sidebars,
the Conclusion, and the TL;DRs. Images/figures are not present (this chapter is largely
code-and-prose). License noted as CC BY-NC-ND 4.0.
