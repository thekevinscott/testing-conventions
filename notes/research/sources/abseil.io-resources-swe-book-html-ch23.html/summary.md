# Summary

- **Title:** Continuous Integration (Chapter 23 of *Software Engineering at Google*)
- **Authors:** Rachel Tannenbaum (edited by Lisa Carey); sidebars by Titus Winters ("CI Is
  Alerting") and Adam Bender ("TAP: Google's Global Continuous Build")
- **URL:** https://abseil.io/resources/swe-book/html/ch23.html
- **Date:** Not dated on page (book published 2020); fetched 2026-06-27
- **Venue:** *Software Engineering at Google* (the "SWE Book"), O'Reilly / Google
- **Source type:** (book chapter)

## What the source claims

The chapter presents Google's view of Continuous Integration (CI), starting from a classic
definition and extending it for large, microservice-based, rapidly evolving systems:

> "*Continuous Integration (2)*: the continuous assembling and testing of our entire complex
> and rapidly evolving ecosystem."

From a testing perspective, CI decides *which* tests to run *when* in the dev/release
workflow and *how* to compose the system under test (SUT) at each point, "balancing concerns
like fidelity and setup cost."

**Fast feedback loops.** "the cost of a bug grows almost exponentially the later it is
caught." Feedback loops are enumerated fastest-to-slowest (edit-compile-debug locally →
presubmit → post-submit → staging/QA → internal users → external users/press). Canarying,
experiments, and feature flags are described as feedback/risk-reduction mechanisms; *version
skew* ("a state of a distributed system in which it contains multiple incompatible versions
of code, data, and/or configuration") is introduced as a recurring hazard. Feedback should
be both **accessible** (unified test reporting, open logs excluding PII, flake classification)
and **actionable**.

**Automation: Continuous Build (CB) and Continuous Delivery (CD).** CB "integrates the latest
code changes at head and runs an automated build and test"; "breaking the build" includes
breaking tests. Two versions of head exist: *true head* (latest commit) and *green head*
(latest CB-verified). CD "continuously assembles release candidates" (RCs) — "A cohesive,
deployable unit ... assembled of code, configuration, and other dependencies that have passed
the continuous build." Configuration must be promoted with code: "a large percentage of
production bugs are caused by 'silly' configuration problems."

**Continuous Testing.** As a change shifts right, it is "subjected to progressively
larger-scoped automated tests." Presubmit is not enough because running all tests on
presubmit "is too expensive," flaky/unstable tests block engineers, and *mid-air collisions*
("two changes that touch completely different files ... cause a test to fail") happen "most
days at our scale." Rule of thumb: run "only fast, reliable ones" on presubmit (typically
small/unit tests for the project where the change happens), accept some coverage loss, and
catch the rest post-submit with rollbacks. RCs are re-tested (sanity check, auditability,
cherry picks, emergency pushes), and production gets the same suite (*probers*) — a "defense
in depth" approach.

**CI Is Alerting** (Winters sidebar): frames CI as the "left shift" of production alerting;
both aim to "identify problems automatically, as soon as possible." Brittle/flaky tests are
analogized to spurious alerts: "If it isn't actionable, it shouldn't be alerting. If it isn't
actually violating the invariants of the SUT, it shouldn't be a test failure." It argues
100% green is as expensive as 100% uptime, and that blanket "nobody can commit if CI isn't
green" policies are "probably misguided."

**CI Challenges:** presubmit optimization, culprit finding / failure isolation, resource
constraints, failure management (disabling/tracking broken or flaky end-to-end tests), and
test instability. **Hermetic testing** is offered as a mitigation: "tests run against a test
environment ... that is entirely self-contained (i.e., no external dependencies like
production backends)," giving greater determinism and isolation. *Record/replay* is described
as a cheaper alternative to large sandboxed stacks but one that "leads to brittle tests"
(false positives from over-caching, false negatives from under-caching).

**TAP** (Bender sidebar): Google's global continuous build over the monorepo. Teams provide a
fast presubmit subset; a passing presubmit has a "very high likelihood (95%+)" of passing the
rest, so changes are optimistically integrated. Each team has a **Build Cop** who fixes
breakages, with **rollback** as the primary tool ("Any change to Google's codebase can be
rolled back with two clicks!").

**Case study — Google Takeout:** four scenarios show CI evolution: (1) moving config/startup
tests to presubmit and end-to-end tests to a two-hourly post-submit; (2) refactoring
indecipherable logs into a parameterized UI with debuggable failure links; (3) running the
same suite against prod to isolate failures; (4) "keeping it green" via tagged test
disablement with auto-cleanup ("MTTCU: mean time to clean up").

## Method / evidence type

Experience report and practitioner guidance from Google, organized around concepts,
best-practice rules, two expert sidebars, and one longitudinal application case study
(Google Takeout). Evidence is qualitative organizational experience plus a few reported
operational metrics; no controlled experiment or dataset is presented.

## Numbers recorded

Figures quoted verbatim from the transcript:
- Presubmit pass implies "very high likelihood (95%+) of passing the rest of the tests."
- TAP: "more than 50,000 unique changes *and* running more than four billion individual test
  cases" per day; evaluates "more than one [change] a second."
- "the average wait time to submit a change is around 11 minutes."
- Difference in wait time between a change triggering "100 tests and one that triggers 1,000
  can be tens of minutes on a busy day."
- Hermetic Google Assistant: previously "more than 50 code changes bypass and ignore the test
  results" on bad days; moving presubmit to hermetic "cut the runtime by a factor of 14, with
  virtually no flakiness."
- DisplayAds "starts about four hundred servers from scratch on every presubmit."
- Assistant failure-isolation cost reduced from *O*(N^2) to *O*(N) via "hotswapping."
- Takeout grew from one product to "at least 10 other Google products," later "more than
  *90*."
- Takeout Scenario 1: sandboxed presubmit tests "prevented 95% of broken servers from bad
  configuration and reduced nightly deployment failures by 50%"; moving end-to-end tests to
  two-hourly post-submit "cut the 'culprit set' by 12 times."
- Takeout Scenario 2: friendlier UI/debugging "reduced the Takeout team's involvement in
  debugging client (product plug-in) test failures by 35%."
- Footnote on uptime targets: "Pick something like 99.9% or 99.999%."

## Scope, limitations, and gaps

- Practices are specific to Google's monorepo, trunk-based development, and bespoke tooling
  (TAP, Forge, Blaze/Bazel, Borg, Rapid); the chapter itself flags ("But I Can't Afford CI")
  that Google has more resources than a typical startup.
- The reported metrics are operational figures embedded in narrative, not outputs of a
  controlled study; no methodology, baselines, or confidence intervals are given.
- "CI Is Alerting" is explicitly framed as a new, still-developing analogy ("this insight is
  new, and we're still figuring out how to fully draw parallels").
- Figures 23-1 through 23-5 are referenced but are images not rendered in the text capture.

## Capture status

`transcript.md` (curl, `capture_status: ok`) contains the **full chapter text**: all prose
sections, the "CI Is Alerting" and "TAP" sidebars, the Hermetic Google Assistant box, the
four-scenario Google Takeout case study, the Conclusion, TL;DRs, and all footnotes (1–12).
Inline figures (`seag_2301`–`seag_2305`) appear only as image references, not content.
License noted as CC BY-NC-ND 4.0.
