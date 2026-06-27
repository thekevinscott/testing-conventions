# Relevance — To Mock or Not To Mock? An Empirical Study on Mocking Practices

**Verdict:** relevant — peer-reviewed empirical study (MSR 2017) that measures, across four
Java systems and a 105-professional survey, *when* developers mock test dependencies and
*why*, plus the maintenance challenges mocking creates. It is the primary evidence base for
any mocking guidance in a testing strategy.

## Salient sections

- **Mock by architectural role, not by coupling/complexity.** "the usage of mocks is highly
  dependent on the responsibility and the architectural concern of the class." Dependencies
  touching external resources are mocked most: Database 72% mocked, Web service 69%, External
  dependencies 68% (Figure 2, N=2,178) → for LLM-authored code, default to mocking the
  I/O/infrastructure boundary, not domain logic.
- **Don't mock domain logic or the unit under test.** Domain objects are mocked only 36% of
  the time (64% not), Java libraries 7%, test support 6%; and "over ~38,000 analyzed
  dependencies the unit under test is never mocked." → the gate for an LLM-authored test
  should flag mocking of domain rules or self as a smell.
- **Coupling/complexity metrics do NOT predict mock decisions.** Mean coupling of mocked
  classes 5.89 vs non-mocked 7.131 (Cliff's δ=−0.121, negligible); complexity 10.58 vs 16.42
  (δ=−0.166). "We conjecture that the chosen code metrics are not enough to predict whether a
  class should be mocked." → a strategy can't auto-decide mocking from static metrics; the
  decision is semantic (what the dependency *is*).
- **Primary cost of mocking = drift from the real class.** "maintaining the behavior of the
  mock compatible with the behavior of original class is hard and ... mocking increases the
  coupling between the test and the production code," and "mocks may hide important design
  problems." → over-mocked LLM tests risk passing while diverging from real behavior; this
  argues for limiting mock surface and preferring real collaborators where setup is cheap.
- **Survey corroboration (N=105):** always/almost-always mock Web services ~82%, External
  dependencies ~79%, Databases ~71%; native libraries never/almost-never mocked by 82%.
  Slowness alone is "not an important factor." → reinforces a boundary-mocking default.

## Evidence weight

- **Study type:** mixed-methods empirical — static-analysis mining (MockExtractor over
  Mockito suites) + manual classification (95% CI, 5% error) + 3 developer interviews +
  105-professional survey + a Mockito core-developer validation interview.
- **Sample/scope:** four Java systems (Sonarqube, Spring Framework, VRaptor open-source;
  Alura industrial); >2,000 dependencies manually analyzed; 4,419 test units (25.39% contain
  a mock).
- **Caveats (author-stated):** single language (Java) and single framework (Mockito), spies
  out of scope; static extraction misses dynamically created mocks (counted as non-mocked);
  one researcher per class (89% agreement on cross-validated subset); survey self-selection
  bias. Findings are about developer *practice/preference*, not a controlled measurement of
  fault-detection outcomes — so it grounds *what to mock*, not a proven bug-finding delta.
  No LLM-generated code studied; transfer to LLM-authored code is by analogy of the same
  language/architecture, not direct measurement.
