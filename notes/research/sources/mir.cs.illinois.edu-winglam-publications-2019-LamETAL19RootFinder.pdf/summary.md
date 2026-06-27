# Summary

- **Title:** Root Causing Flaky Tests in a Large-Scale Industrial Setting
- **Authors:** Wing Lam (University of Illinois at Urbana-Champaign); Patrice Godefroid, Suman Nath, Anirudh Santhiar, Suresh Thummalapenta (Microsoft Corporation)
- **URL:** https://mir.cs.illinois.edu/winglam/publications/2019/LamETAL19RootFinder.pdf
- **Date:** 2019 (ISSTA '19, July 15–19, 2019, Beijing, China)
- **Venue:** Proceedings of the 28th ACM SIGSOFT International Symposium on Software Testing and Analysis (ISSTA '19); DOI 10.1145/3293882.3330570
- **Source type:** Full peer-reviewed conference paper (PDF captured)

## What the source claims

Two-part paper: (1) a quantitative study of flaky-test prevalence at Microsoft, plus a
public dataset of execution logs; and (2) a preliminary tool, **RootFinder**, with an
end-to-end framework (using **Torch** instrumentation) that compares logs of passing vs.
failing runs to suggest the method calls responsible for flakiness. Central claims: distinct
flaky tests can be few while the share of builds they break is large; flakiness is hard to
reproduce locally; and log-differencing can help root-cause most flaky-test categories.

Verbatim quotes:

> "over a one-month period monitoring of five software projects, we observed that 4.6% of
> all individual test cases are flaky."

> "When we re-run flaky tests locally 100 times, we find that 86% of them are only flaky in
> the CI pipeline."

> "although the number of distinct flaky tests is comparatively low, the number of
> validation runs that could fail due to flaky tests is high, emphasizing the problematic
> nature of flaky tests to developers."

> "The predicates outputted by RootFinder can aid the debugging efforts of nine out of ten
> categories of flaky tests mentioned in a survey [34]."

> "there is no correlation between the number of flaky tests and the number of builds that
> fail due to flaky tests"

Cited (not the authors' own measurement), from Micco's Google keynote:

> "Micco observed that 1.5% of all test runs in Google's CI pipeline are flaky, and almost
> 16% of 4.2 million individual tests fail independently of changes in code or tests."

## Method / evidence type

- Production data from Microsoft's **CloudBuild** CI (≈1200 projects; ≈350 million unit
  tests/day). Flaky tests identified by CloudBuild's automatic rerun-on-failure.
- Prevalence study over **30 days** across **five** anonymized projects (Table 1).
- **Torch** instrumentation logs runtime properties (call info, timestamps, return values,
  object/thread ids) for all API calls; each flaky test is run **100 times** locally to
  collect passing and failing logs.
- **RootFinder** evaluates boolean predicates (Relative, Absolute, Exception, Order, Slow,
  Fast) per log line and classifies them (Inconsistent-in-passing / -in-failing,
  Consistent-and-matching, Consistent-but-different) to surface root causes.
- Five qualitative case studies (Time, Randomness, Async Wait, Concurrency, Resource Leak).
- A developer survey (58 Microsoft developers; an 18-developer follow-up).

## Numbers recorded

Prevalence (Table 1, five projects):
| Project | # Tests | # Builds | Test Executions | Flaky Test Failures | Distinct Flaky Tests | Builds w/ Flaky Failures | % Builds w/ Flaky Failures |
|---|---|---|---|---|---|---|---|
| ProjA | 26,404 | 302 | 6,670,299 | 6,106 | 2,165 | 43 | 14% |
| ProjB | 5,675 | 430 | 2,433,452 | 537 | 125 | 224 | 52% |
| ProjC | 23,651 | 575 | 4,596,490 | 3,530 | 190 | 173 | 30% |
| ProjD | 5,693 | 741 | 1,449,233 | 328 | 98 | 126 | 17% |
| ProjE | 3,390 | 1,823 | 786,898 | 1,564 | 286 | 429 | 24% |

- Their measured prevalence: **4.6%** of all individual tests flaky; on average **27.4%** of
  builds exhibit flaky tests.
- Reproducibility: **86%** flaky only in CI (after 100 local reruns). Of **315** flaky tests
  matching the instrumentation criteria, only **44** were reproducible (passing+failing logs
  in 100 runs); **97** had all runs pass; **174** had all 100 runs fail. (Also stated: "Of
  311 tests in our dataset that failed on the cloud, we could not reproduce the failure
  locally in 97 cases.")
- Instrumentation effect: of **59** randomly sampled tests run 100x with and without Torch,
  **2** flaky only with Torch, **3** only without.
- Per-test-run averages: **335k** method calls, **5** threads, **55,418** objects.
- Dataset: **44** flaky tests, **22** software projects, **18** Microsoft products; **100**
  execution traces each.
- RootFinder aids **9 of 10** flaky-test categories from the cited survey.

Table 2 (all 44 flaky tests): Median — duration 5s, 6% failed executions/test, 2.5k method
calls, 248 unique method calls, 3 threads, 637 objects. Average — 45s, 28%, 335k, 335, 5,
55,418. **80%** of tests use more than one thread.

Case studies (runtime / predicates output / rank of useful predicate):
- Time: fails ~29% (others up to 88%); 11s; 1163 predicates; useful predicate ranked **81**.
- Randomness: fails **91/100**; 2s; 408 predicates; useful predicate ranked **1st**.
- Async Wait: fails **99/100**; 1s; 868 predicates; useful predicate ranked **17**.
- Concurrency: 126s; 127,187 predicates; useful predicate ranked **3231**.
- Resource Leak: framework currently **cannot** root-cause (relies on garbage-collector timing).

Cited related-work figures: Google spends ≈2–16% of testing budget rerunning flaky tests;
Labuschagne et al. 12.8% of 935 builds (61 projects, Travis CI) failed due to flaky tests;
Palomba and Zaidman found 8829/19532 (45%) JUnit tests flaky across 18 projects (≤10 reruns);
Lam et al. found 388 flaky tests in 683 projects; Thorve et al. found 13% of 77 commits (29
Android projects) simply skipped/removed flaky tests.

## Scope, limitations, and gaps

- **Preliminary tool**, explicitly framed to "encourage more work"; not a complete solution.
- RootFinder implementation supports only **unsigned, managed** code (e.g., C#) and is
  tailored to the **MsTest** framework; relies on a predefined set of nondeterministic method
  calls (developer-tunable).
- Only **44 / 315** flaky tests were reproducible — the analysis runs on a small reproducible
  subset, and nondeterminism means results may not replicate exactly on re-experiment (stated
  threat to validity).
- Concurrency case shows poor predicate ranking (3231) without domain knowledge; Resource
  Leak unsupported — coverage of "9 of 10 categories" is in theory and partly relies on
  developer-supplied domain knowledge.
- Five-project prevalence study is Microsoft-specific (CloudBuild); generalization beyond is
  not claimed.

## Capture status

`transcript.md` is the **full paper PDF** (curl, `ok`): abstract, introduction, experience
section with Table 1, the Torch/RootFinder framework, predicate definitions, Table 2, the
five case studies (Figures 2–8 as code listings), lessons learned, and related work. Figures
that are plots/log fragments are described in prose; their raw images are not rendered.
