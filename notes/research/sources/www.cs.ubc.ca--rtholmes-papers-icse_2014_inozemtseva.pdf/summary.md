# Summary

- **Title:** Coverage Is Not Strongly Correlated with Test Suite Effectiveness
- **Authors:** Laura Inozemtseva, Reid Holmes
- **URL:** https://www.cs.ubc.ca/~rtholmes/papers/icse_2014_inozemtseva.pdf
- **Date:** 2014
- **Venue:** ICSE '14 (Int'l Conference on Software Engineering), May 31–June 7, 2014, Hyderabad, India
- **Source type:** (peer-reviewed study; full paper PDF captured)

## What the source claims

Investigates whether code coverage is a good proxy for a test suite's fault-detection
effectiveness on large Java programs, controlling for suite size. Central claim: when the
number of test cases is held constant, the correlation between coverage and effectiveness
drops to low/moderate; much of the apparent coverage–effectiveness relationship is driven
by suite size. Stronger coverage types do not add insight. Coverage is useful for finding
under-tested code but should not be used as a quality target.

Verbatim quotes:

> "We found that there is a low to moderate correlation between coverage and effectiveness
> when the number of test cases in the suite is controlled for. In addition, we found that
> stronger forms of coverage do not provide greater insight into the effectiveness of the
> suite. Our results suggest that coverage, while useful for identifying under-tested parts
> of a program, should not be used as a quality target because it is not a good indicator of
> test suite effectiveness."

> "we generated 31,000 test suites for five systems consisting of up to 724,000 lines of
> source code."

> "These results imply that high levels of coverage do not indicate that a test suite is
> effective. Consequently, using a fixed coverage value as a quality target is unlikely to
> produce an effective test suite."

> "The type of coverage used had little impact on the strength of the correlation."

> "In general, there is a low to moderate correlation between the coverage of a test suite
> and its effectiveness when its size is controlled for."

On the size confound (Answer 3):

> "After this drop, the correlation typically ranges from low to moderate, meaning it is not
> generally safe to assume that effectiveness is correlated with coverage."

On standards implications:

> "The FAA standard DO-178B ... requires the use of MC/DC adequate suites ...; however, our
> results suggest that this requirement may increase expenses without necessarily increasing
> quality."

## Method / evidence type

- Five large open-source Java subject programs: **Apache POI, Closure Compiler, HSQLDB,
  JFreeChart, Joda Time** (selected for size ~100k SLOC, ~1,000+ test methods, Ant + JUnit).
- Generated **31,000 test suites** by randomly sampling test methods (without replacement)
  at fixed sizes (3, 10, 30, 100, 300, 1000, 3000 methods); 1,000 suites per size per program.
- **Mutation testing** with the tool PIT to measure effectiveness (fraction of non-equivalent
  mutants killed); any mutant the full master suite could not kill was assumed equivalent.
- Coverage measured with **CodeCover**: statement, decision, and modified condition coverage.
- Two effectiveness metrics: raw and normalized kill score.
- Correlations measured with **Kendall τ** (non-parametric); size correlations via R's `lm`
  (adjusted r²). Significance reported at the 99.9% level for τ tables.

## Numbers recorded

Subject programs (Table 2):

| Property | Apache POI | Closure | HSQLDB | JFreeChart | Joda Time |
|---|---|---|---|---|---|
| Total Java SLOC | 283,845 | 724,089 | 178,018 | 125,659 | 80,462 |
| Test methods | 1,415 | 7,947 | 628 | 1,764 | 3,857 |
| Statement coverage (%) | 67 | 76 | 27 | 54 | 91 |
| Decision coverage (%) | 60 | 77 | 17 | 45 | 82 |
| MC coverage (%) | 49 | 67 | 9 | 27 | 70 |
| Number of mutants | 27,565 | 30,779 | 50,302 | 29,699 | 9,552 |
| Equivalent mutants (%) | 35 | 11 | 0.4 | 21 | 11 |

Size vs effectiveness (RQ1): adjusted r² for normalized effectiveness ranged 0.26 to 0.97
("implying that the correlation coefficient r ranges from 0.51 to 0.98"); non-normalized r²
0.69 to 0.99 — "a moderate to very high correlation between ... effectiveness and size."

Coverage vs normalized effectiveness, size ignored (Table 3, Kendall τ):

| Project | Statement | Decision | Mod. Cond. |
|---|---|---|---|
| Apache POI | 0.75 | 0.76 | 0.77 |
| Closure | 0.83 | 0.83 | 0.84 |
| HSQLDB | −0.35 | −0.35 | −0.35 |
| JFreeChart | 0.50 | 0.53 | 0.53 |
| Joda Time | 0.80 | 0.80 | 0.80 |

Coverage vs non-normalized effectiveness, size ignored (Table 4, Kendall τ): 0.79–0.95
across projects/coverage types.

Coverage vs effectiveness, size fixed (RQ3): "Controlling for suite size always lowered the
correlation." Joda Time dropped "to essentially zero"; Apache POI (non-normalized) dropped
from 0.94 to a range of 0.46 to 0.85.

Correlation between coverage types (Table 5, all suites): Statement/Decision τ=0.92
(Pearson 0.99); Decision/MCC τ=0.91 (0.98); Statement/MCC τ=0.92 (0.97).

Gopinath et al. context cited: "only 729 of the 1,254 open source Java projects they
initially considered, or 58%, had test suites at all."

## Scope, limitations, and gaps

Stated by the authors (Threats to Validity, Section 6):

- **Construct:** effectiveness is estimated via mutant kills, not real faults; up to 35% of
  mutants were classified as equivalent (assumed-equivalent shortcut), which "introduces a
  threat" if those mutants are not a random subset.
- **Internal:** conclusions depend on Kendall τ; ties were checked (worst case 4.6% of
  comparisons tied) and judged negligible. "Since we have studied correlations, we cannot
  make any claims about the direction of causality."
- **External (six threats):** results may not hold where faults are hard to detect; the
  relationship may only appear at very high coverage (which generated suites rarely reached);
  per-class variation possible; class size not controlled; subjects met narrow inclusion
  criteria (Java, Ant, JUnit, ~1,000 tests) so are "fairly similar"; all subjects open
  source and "still not large by industrial standards."
- No dataflow coverage measured (tool limitations); planned for future work.
- Future work: confirm findings using real faults; longitudinal study.

## Capture status

`transcript.md` is the full paper PDF (`transport: curl`, `capture_status: ok`). It
contains the abstract, full body, Tables 1–5, and references. Figures 2–4 are scatter/box
plots that extracted only as point-cloud artifacts; the numeric values above were taken
from the prose, the in-text r²/τ figures, and the data tables.
