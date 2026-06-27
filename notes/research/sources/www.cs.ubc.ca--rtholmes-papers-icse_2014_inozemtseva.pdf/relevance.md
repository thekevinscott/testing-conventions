# Relevance — Coverage Is Not Strongly Correlated with Test Suite Effectiveness

**Verdict:** relevant — peer-reviewed empirical study (ICSE 2014) that, using mutation
testing over 31,000 generated suites on five large Java systems, measures whether code
coverage predicts a suite's fault-detection effectiveness once suite size is controlled. It
is primary evidence on whether coverage is a sound *gate* for test quality.

## Salient sections

- **Coverage is a weak proxy for effectiveness once size is held constant.** "there is a low
  to moderate correlation between coverage and effectiveness when the number of test cases in
  the suite is controlled for ... coverage ... should not be used as a quality target." →
  for LLM-authored code, a coverage % threshold is an unreliable acceptance gate; mutation
  kill rate is the stronger effectiveness signal used here.
- **Apparent coverage–effectiveness link is largely a size confound.** Size vs effectiveness
  adjusted r² 0.26–0.97 (non-normalized 0.69–0.99); controlling for size "always lowered the
  correlation," with Joda Time dropping "to essentially zero." → an LLM that inflates
  coverage by adding many shallow tests can hit a target without improving fault detection.
- **Stronger coverage criteria add no insight.** "stronger forms of coverage do not provide
  greater insight"; statement/decision/MC coverage track each other (τ≈0.91–0.92, Pearson
  0.97–0.99). → no value in mandating MC/DC-style criteria over statement coverage for the
  acceptance gate.
- **Coverage's legitimate use is diagnostic, not a target.** "useful for identifying
  under-tested parts of a program, [but] should not be used as a quality target." → use
  coverage to find untested LLM-written code, then judge the *tests* by mutation/fault
  detection.
- **Standards implication:** DO-178B's MC/DC requirement "may increase expenses without
  necessarily increasing quality." → caution against high-cost coverage mandates as the gate.

## Evidence weight

- **Study type:** controlled empirical study; effectiveness = fraction of non-equivalent
  mutants killed (PIT), coverage via CodeCover (statement/decision/modified-condition),
  correlations via Kendall τ and adjusted r².
- **Sample/scope:** five open-source Java systems (Apache POI, Closure Compiler, HSQLDB,
  JFreeChart, Joda Time; up to 724k SLOC); 31,000 randomly sampled suites across fixed sizes
  (3–3000 methods, 1,000 suites per size per program).
- **Caveats (author-stated):** effectiveness estimated via mutants, not real faults (up to
  35% mutants assumed equivalent); correlational only — "we cannot make any claims about the
  direction of causality"; subjects met narrow inclusion criteria (Java/Ant/JUnit) so are
  "fairly similar" and "still not large by industrial standards"; relationship might re-emerge
  only at very high coverage rarely reached by the generated suites. Not LLM-specific, but
  language-agnostic in mechanism — directly informs the coverage-vs-mutation gate choice.
