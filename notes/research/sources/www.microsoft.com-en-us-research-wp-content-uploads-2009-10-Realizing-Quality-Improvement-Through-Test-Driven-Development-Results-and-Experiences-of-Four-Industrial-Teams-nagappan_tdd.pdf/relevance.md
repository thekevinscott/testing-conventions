# Relevance — Realizing Quality Improvement Through Test-Driven Development: Results and Experiences of Four Industrial Teams

**Verdict:** relevant — peer-reviewed empirical industrial study (Empirical Software
Engineering 2008) measuring the defect-density effect of TDD across four Microsoft/IBM teams
versus comparable non-TDD projects. It is primary evidence on TDD's effectiveness, the
workflow this project's own AGENTS.md mandates (red tests before implementation).

## Salient sections

- **TDD substantially lowers pre-release defect density.** "the pre-release defect density of
  the four products decreased between 40% and 90% relative to similar projects that did not
  use the TDD practice" — TDD-team densities 0.61W, 0.38X, 0.24Y, 0.09Z (Table 3; ≈39/62/76/91%
  reductions). → grounds a red-test-first workflow as a measurably effective quality
  strategy, including for LLM-authored code where the LLM writes failing tests before code.
- **Cost is upfront time, offset by quality.** "the teams experienced a 15–35% increase in
  initial development time," which authors say "is offset by the ... reduced maintenance costs
  due to the improvement in quality." → the trade is more authoring effort now for fewer
  defects later; relevant when an LLM bears the authoring cost cheaply.
- **High unit-test coverage accompanied the gains.** Block coverage from unit tests: IBM 95%,
  Windows 79%, MSN 88%, VS 62%; test/source KLOC 0.39–0.89 (Table 2). → substantial test mass
  is part of the observed effect (though see Inozemtseva: coverage is correlate, not the gate).
- **Discipline matters / regression on lapse.** A later IBM release saw defect density rise
  "temporally" when some members skipped running the unit tests. → the benefit depends on the
  tests actually being authored and run every cycle, not merely existing.
- **Corroborating prior experiment cited:** George & Williams (24 professionals) — TDD
  programmers "passed 18% more functional black-box test cases" but "took 16% more time."

## Evidence weight

- **Study type:** four in-vivo industrial case studies; defect density (defects/KLOC) mined
  from IBM/Microsoft bug databases; each TDD team compared to a non-TDD team under the same
  senior manager. Analyzed post hoc (teams unaware they would be studied).
- **Sample/scope:** IBM device drivers + Microsoft Windows, MSN, Visual Studio; team sizes
  5–9; languages Java, C/C++, C#; 6–155 KLOC source.
- **Caveats (author-stated):** case studies, not controlled experiments — "a family of case
  studies is likely not to yield statistically significant results"; the development-time
  increase is a *subjective management estimate*, not measured; maintenance-cost offset is
  asserted, not quantified; project comparability is imperfect (esp. IBM new-TDD vs legacy
  enhancement); generalizability not assured. Pre-LLM era — transfers to LLM-authored code by
  process analogy (the workflow), not by direct measurement on LLM code.
