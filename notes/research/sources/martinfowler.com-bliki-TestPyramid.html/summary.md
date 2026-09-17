# Summary

- **Title:** Test Pyramid
- **Author / org:** Martin Fowler (martinfowler.com / Thoughtworks)
- **URL:** https://martinfowler.com/bliki/TestPyramid.html
- **Date:** 1 May 2012 (revisions: 7 Aug 2016 illustration; 15 Nov 2017 etymology)
- **Venue:** Personal site (martinfowler.com) "bliki", "testing" tag
- **Source type:** (practitioner article / definitional essay) by Martin Fowler — opinion/definitions, not a study

## What the source claims

The test pyramid is a heuristic for balancing a test portfolio: many fast low-level unit
tests, far fewer slow high-level UI/end-to-end tests, with a middle service-layer tier.
Opinion/advice, grounded in the author's experience; not an experiment.

Verbatim quotes:

> "Its essential point is that you should have many more low-level UnitTests than high level
> BroadStackTests running through a GUI."

> "tests that run end-to-end through the UI are: brittle, expensive to write, and time
> consuming to run."

> "The pyramid also argues for an intermediate layer of tests that act through a service
> layer of an application, what I refer to as SubcutaneousTests."

> "I always argue that high-level tests are there as a second line of test defense. If you
> get a failure in a high level test, not just do you have a bug in your functional code,
> you also have a missing or incorrect unit test."

> "before fixing a bug exposed by a high level test, you should replicate the bug with a
> unit test. Then the unit test ensures the bug stays dead."

On a stated caveat / exception:

> "The pyramid is based on the assumption that broad-stack tests are expensive, slow, and
> brittle compared to more focused tests, such as unit tests. While this is usually true,
> there are exceptions. If my high level tests are fast, reliable, and cheap to modify -
> then lower-level tests aren't needed."

On record-playback UI tools (becoming an "ice-cream cone"):

> "Record-playback tools are almost always a bad idea for any kind of automation, since they
> resist changeability and obstruct useful abstractions."

On the debate over the shape:

> "Some writers argue that the pyramid isn't a good test distribution, preferring more
> integration tests and few unit tests. But difference is probably illusory due to different
> definitions of 'unit test'."

## Method / evidence type

Opinion / definitional essay with practitioner reasoning. Cites external corroborating
material (Google Testing Blog against end-to-end tests; Adrian Sutton / LMAX for the
counter-case) but presents no data of its own.

## Numbers recorded

None. No quantitative measurements. Etymology dates are the only figures: Mike Cohn's 2009
book "Succeeding with Agile" (named it the "Test Automation Pyramid"); drawn in conversation
with Lisa Crispin 2003–4 and described at a scrum gathering in 2004; Jason Huggins
independently arrived at the same idea around 2006.

## Scope, limitations, and gaps

- Heuristic, not a measured optimum; the author himself flags exceptions (fast/reliable
  high-level tests can displace lower-level ones).
- "Conflating end-to-end tests, UI tests, and customer facing tests" is called out as a
  common error — the pyramid's layers are treated as orthogonal characteristics.
- The "unit test" definition ambiguity is acknowledged as the source of disagreement about
  the ideal shape.

## Capture status

`transcript.md` is the **full bliki entry** (curl, `ok`): main text, footnotes 1–2,
Further Reading, Acknowledgements, Etymology, and Revisions. The pyramid illustration is an
image link only (not rendered). Remainder is site chrome/nav.
