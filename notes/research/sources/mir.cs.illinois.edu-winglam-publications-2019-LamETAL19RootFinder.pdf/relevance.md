# Relevance — Root Causing Flaky Tests in a Large-Scale Industrial Setting (Lam, Godefroid, Nath, Santhiar, Thummalapenta)

**Verdict:** relevant — full peer-reviewed empirical study (ISSTA '19) measuring flaky-test prevalence and reproducibility at Microsoft scale, plus a log-differencing root-causing tool (RootFinder). Directly addresses the flaky-tests category named in the rubric with primary industrial data.

## Salient sections

- **Flaky-test prevalence (Experience study, Table 1).** Over a 30-day window across five projects, "**4.6%** of all individual test cases are flaky," and on average **27.4%** of builds exhibit a flaky failure (per-project 14–52%). → Flakiness is a first-order reliability problem any test suite must budget for, including auto-generated suites.
- **Distinct flaky tests ≠ build damage.** "there is no correlation between the number of flaky tests and the number of builds that fail due to flaky tests" — a few flaky tests can break a large share of builds. → Optimizing a strategy means targeting the *high-churn* flaky tests, not just counting them.
- **Flakiness is mostly a CI phenomenon (reproducibility).** "When we re-run flaky tests locally 100 times, we find that **86%** of them are only flaky in the CI pipeline." Of 315 candidate flaky tests, only 44 were reproducible locally (passing+failing logs); 97 always passed, 174 always failed. → Local reruns are a weak gate; flakiness detection/quarantine must run in the CI environment.
- **Root causes are concrete and recurring (case studies).** Demonstrated categories: Time, Randomness, Async Wait, Concurrency, Resource Leak; **80%** of the 44 flaky tests use more than one thread. RootFinder's predicate differencing surfaces a useful root-cause predicate ranked 1st (Randomness) to thousands-deep (Concurrency, rank 3231); Resource Leak (GC-timing) it cannot root-cause. → Nondeterminism sources (time, randomness, concurrency, async waits) are the prime targets for stabilizing a suite — pertinent because LLM-generated tests readily introduce timing/async assumptions.
- **Cited corroboration (not authors' own).** Micco/Google: 1.5% of test runs flaky, ~16% of 4.2M tests fail independent of code changes; Google spends ~2–16% of testing budget rerunning flaky tests; Palomba & Zaidman: 45% of JUnit tests flaky across 18 projects. → Independent magnitude checks on the flakiness tax.

## Salient sections — strategy takeaway

For LLM-authored code, the evidence argues a robust strategy must (a) detect flakiness in the CI environment, not via local reruns; (b) prioritize the flaky tests that break the most builds; and (c) treat nondeterminism sources (time, randomness, concurrency, async, resource leaks) as the dominant failure modes to design out.

## Evidence weight

- **Study type:** Empirical industrial measurement (Microsoft CloudBuild CI, ~1,200 projects, ~350M unit tests/day) + Torch instrumentation + RootFinder predicate analysis + developer survey (58, then 18). Prevalence over 30 days, five anonymized projects.
- **Scope/sample:** Prevalence on 5 projects; root-causing dataset only **44** reproducible flaky tests across 22 projects / 18 products, 100 traces each.
- **Caveats / not on LLM code:** General SE study of **human-written** (largely C#/MsTest) code — flakiness findings transfer to LLM-authored code by analogy, not direct measurement. RootFinder is explicitly **preliminary**, supports only unsigned/managed code, relies on a tunable list of nondeterministic calls, and depends partly on developer domain knowledge (Concurrency poorly ranked, Resource Leak unsupported). Prevalence is Microsoft-specific; nondeterminism means results may not replicate exactly.
