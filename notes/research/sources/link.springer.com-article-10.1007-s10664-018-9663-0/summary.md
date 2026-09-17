# Summary

- **Title:** Mock objects for testing java systems — Why and how developers use them, and how they
  evolve
- **Authors:** Davide Spadini, Maurício Aniche, Magiel Bruntink, Alberto Bacchelli
- **URL:** https://link.springer.com/article/10.1007/s10664-018-9663-0
- **Date:** Published 06 November 2018; Empirical Software Engineering, Volume 24, pages 1461–1498
  (2019). Open access. (Page metadata: 19k accesses, 57 citations.)
- **Source type:** (peer-reviewed study) — Empirical Software Engineering journal article;
  mixed-methods empirical study. Extends the authors' MSR 2017 paper "To Mock or Not To Mock?"

## What the source claims

The study characterises **how, why, and when developers use mock objects** in Java, and how mocks
evolve. Central findings (abstract, verbatim):

> "Our study reveals that the usage of mocks is highly dependent on the responsibility and the
> architectural concern of the class. Developers report to frequently mock dependencies that make
> testing difficult (e.g., infrastructure-related dependencies) and to not mock classes that
> encapsulate domain concepts/rules of the system."

> "Among the key challenges, developers report that maintaining the behavior of the mock
> compatible with the behavior of original class is hard and that mocking increases the coupling
> between the test and the production code."

> "we observed that mocks mostly exist since the very first version of the test class, and that
> they tend to stay there for its whole lifetime, and that changes in production code often force
> the test code to also change."

The framing trade-off (verbatim):

> "by testing all dependencies together, developers gain realism. By simulating its dependencies,
> developers gain focus."

Key qualitative findings, attributed to interviewees/survey:

- "Mocks are used when the concrete implementation is not simple" (highly coupled, complex,
  slow, or external-resource dependencies).
- "Mocks are not used when the focus of the test is the integration." D1: *"I do not mock when I
  want to test the database itself; I want to make sure that my SQL works. Other than that, we
  mock."*
- "Interfaces are mocked rather than specific implementations."
- "Domain objects are usually not mocked" (unless complex/highly coupled).
- "Native Java objects and libraries are usually not mocked."
- "Database, web services, and external dependencies are slow, complex to set up, and are good
  candidates to be mocked." D2: *"Our database integration tests take 40 minutes to execute, it
  is too much."*

On code quality, a notable null/contradictory result (verbatim):

> "even though developers said that they prefer to mock complex or highly coupled classes, it
> seems not to be the case if we look at code metrics."

Challenge on mocks hiding design problems — D3: *"I always try to use as less mocks as possible,
since in my opinion they hide the real problem. Furthermore, I do not remember a single case in
which I found a bug using mocks."* The Mockito core developer (D4) agreed mocking many
dependencies is "a big red flag that the unit under test is not well designed."

## Method / evidence type

**Mixed-methods empirical study** over four Java systems (3 OSS: Sonarqube, Spring, VRaptor; 1
industrial: Alura), all using Mockito. Two phases:

- Phase 1 (RQ1–RQ3): static analysis via custom tool **MockExtractor** to label dependencies
  mocked/not-mocked; **manual analysis** of a random sample to assign architectural-concern
  categories; **3 developer interviews** (technical leaders) + **survey of 105 developers** +
  discussion with a core Mockito developer (D4).
- Phase 2 (RQ4–RQ5): mine git history of the same four systems to detect when mocks are
  introduced and how Mockito API usage evolves; manual analysis of a change sample.
- Statistics for the code-quality comparison: Wilcoxon rank-sum test (95% confidence) + Cliff's
  delta effect size; non-normality checked via Shapiro-Wilk.

## Numbers recorded

Sampling/agreement:
- Manual-analysis sample: **844 + 1,334 = 2,178** dependencies (95% confidence, 5% error). 116
  raw categories merged into **7**. Inter-rater agreement on the 7 categories: **89%**.
- Survey: **105** respondents (challenges question: 61 responses). Experience: 22% 1–5y, 64%
  6–15y, 14% >15y. Languages: Java 52%, JavaScript 42%, C# 39%. Frameworks: Mockito 53%, Moq
  31%, Powermock 8%. Geography: 66% South America, 21% Europe, 8% North America, 5% India/Africa.
- Mock-change line-matching: cosine-similarity threshold α = 0.71, precision ~73% (73.2%); 13
  Mockito APIs tracked.

RQ1 (what is mocked):
- **4,419** test units analysed; **1,122 (25.39%)** contain at least one mock.
- **38,313** collected dependencies: **35,745 (93.29%)** not mocked, **2,568 (6.71%)** mocked.
- Unique: **11,824** non-mocked, **938** mocked; **650 (70% of ever-mocked)** appear both mocked
  and not-mocked.
- Domain objects: **36%** mocked (no clear trend). Per-project exceptions: databases mocked ~60%
  (Alura, Sonarqube) vs 94% (Spring); domain objects ~30% vs 47% (Sonarqube).
- Survey "always/almost always mock": Web services ~82%, External dependencies ~79%, Databases
  ~71%. Native Java libraries: 82% never/almost never mock. "Always/almost always mock" highly
  coupled classes 48%; difficult-to-set-up classes 45.5%.

RQ4 (when introduced, Table 5, N = 2,935):
- **2,159 (83%)** mocks introduced at test-class inception; **433 (17%)** introduced later
  (~80/20 across all four systems). **343 (13%)** removed afterward.

RQ5 (evolution, Table 6 N = 74,983 API occurrences; change sample Table 7 >300 changes):
- Reasons a mock changes: **production-code-induced 113 + 59 = 172 (51%)** vs **test-code
  improvements 164 (49%)**.
- Production-class **API** changes that forced mock changes: return type 13%, parameter count
  27%, method renamed 30%, class renamed/refactored 30%.
- Production **internal-implementation** changes: internal details changed 73%, method replaced
  19%, class replaced 8%.
- Test-evolution changes: test refactoring 63%, test improvements (e.g. new inputs/corner cases)
  32%.

Code-quality comparison (mocked vs not-mocked classes): differences mostly **negligible**
(Wilcoxon p < 0.001, Cliff's Delta < −0.12); exceptions for high/very-high-tested classes — LOC
Cliff's Delta = −0.37, complexity Cliff's Delta = −0.29.

Motivating example (Sonarqube, Jan 2017): over 5,500 classes, 700k LOC, 2,034 test units; 652 use
mocks, mocking 1,411 unique dependencies.

## Scope, limitations, and gaps

- **Java/Mockito only**, four systems (three OSS + one industrial); generalisability beyond Java
  and beyond projects that "routinely use mock objects" is limited by design.
- Survey skews to South America (66%) and to authors' contacts/mailing lists; recruited via
  Twitter and personal networks (selection bias).
- Only **3 interviews** (one per project, technical leaders) — small qualitative base, augmented
  by one Mockito developer.
- Construct-validity threats the authors flag: MockExtractor (static analysis) misses dynamically
  generated mocks; the cosine-similarity line matcher has ~73% precision; only 13 (oldest, most
  used) Mockito APIs tracked, so "adoption patterns" of newer APIs not studied.
- Code-quality analysis uses four CK-style metrics (CBO, McCabe complexity, LOC, NOM) and finds
  metrics **do not** explain mocking decisions — authors conjecture the metrics are insufficient
  and call for future work.

## Capture status

`transcript.md` is the **full open-access article text** (curl, `capture_status: ok`), not just
the abstract/landing page: abstract, Introduction, Background, full Methodology (RQ1–RQ5), all
Results sections with their inline numbers, Discussion (including the Mockito-developer interview,
code-quality analysis, trade-offs, implications), and Threats to Validity. **Figures and tables
are referenced as image links only** (e.g. Tables 1–7, Figs 1–6) — the underlying per-cell table
values and plotted distributions are not present as text. The transcript is ~1328 lines; sections
after Threats to Validity (related work, conclusion, references) were not all read in full but the
results/numbers above are taken verbatim from the captured body.
