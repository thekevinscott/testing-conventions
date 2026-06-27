# Summary

- **Title:** An empirical study on the usage of mocking frameworks in Apache software foundation
- **Authors:** Lu Xiao, Gengwu Zhao, Xiao Wang, Keye Li, Erick Lim, Chenhao Wei, Tingting Yu, Xiaoyin Wang
- **URL:** https://link.springer.com/article/10.1007/s10664-023-10410-y
- **Date:** Published 23 January 2024; Volume 29, article number 39 (2024)
- **Venue:** Empirical Software Engineering (Springer); open access
- **Source type:** Empirical Software Engineering article (peer-reviewed study)

## What the source claims

A repository-mining study of how mocking frameworks are used across Apache Java
projects, organized around five research questions (RQ1 adoption, RQ2 intensity/type,
RQ3 evolution, RQ4 frequent APIs, RQ5 informal mocking), complemented by a developer
survey. The study does **not** argue for or against mocking; it reports usage.

Verbatim quotes from the abstract / findings:

> "mocking frameworks are widely used in 66% of Apache Java projects, with Mockito,
> EasyMock, and PowerMock being the top three most popular frameworks. Larger-scale
> and more recent projects tend to observe a stronger need to use mocking frameworks."

> "mocking is overall practiced quite selectively in software projects—not all test
> files use mocking, nor all dependencies of a test target are mocked."

> "The top five APIs in each functional type of the three mocking frameworks usually
> take the majority (78% to 100%) of usage in Apache projects."

> "These informal mocking methods point to potential sub-optimal mocking practices that
> could be improved, as well as limitations of existing mocking frameworks."

On a known debate it cites (not the authors' own claim):

> "Some argue that while stubs are often desirable, they are overused through mocking
> frameworks, resulting in fragile tests."

## Method / evidence type

- Repository mining of Apache Software Foundation Java projects. Started from 246 Apache
  Java projects; after excluding projects with no linked git repo (33), no JUnit/test
  cases (4), and JDT-uncompilable projects (16), the analysis dataset is **193 projects**
  (latest GitHub version as of October 2022).
- Source-code parsing via **Eclipse JDT** to resolve API bindings; framework detection by
  searching the keyword "mock" in import namespaces, then manual verification.
- Pair-wise correlation analysis (Python Pandas `corr()`) of project factors; projects
  binned by scale (Java files) and age.
- One snapshot per year per project for evolution analysis (RQ3).
- A developer **survey** (SQ1–SQ10) to triangulate the mined findings.
- Data published at https://github.com/gzhao9/Mock-Apache-Empirical-Study.git

## Numbers recorded

Adoption (RQ1):
- **127 / 193 (66%)** projects use at least one mocking framework.
- Among the 127: Mockito in **98 (77%)**; EasyMock **28%**; PowerMock **12%**. Top 3
  together used in **90%** of projects.
- **47 (37%)** projects use more than one framework (33 use two, 9 use three). Most common
  combinations: Mockito+EasyMock (10 projects), Mockito+PowerMock (7), EasyMock-PowerMock
  (3), Mockito-WireMock (3).
- By scale: **88%** of large-scale, **65%** medium, **34%** small projects use a framework.
- By age: **73%** of projects born after 2010, **63%** 2005–2010, **46%** before 2005.
- By domain: Development & Testing 50%; Networking & IoT 72%; Web Technologies 73%;
  Data Management 83%.

Intensity/type (RQ2):
- In **70%** of projects, <10% of test files use mocking; in **65%**, 5–15% of
  dependencies mocked; in **53%**, fewer than 10 developers worked on mocking.
- **58.5%** of mock-using files create ≤3 mock objects; **13.4%** create >10.
- **36.7%** of non-trivial test files mock only one dependency; **6.5%** mock >10.
- On average **61.1%** of mocked objects isolate external library dependencies (max 100%).

Evolution (RQ3):
- First adoption: 12 projects 2005–2009; 61 projects 2009–2014; 29 projects 2014–2019;
  13 projects 2019–2022. **75 of 115** still active in 2022.
- Duration: 46 projects 6–10 years; 36 projects ≤5 years; 33 projects >10 years.
- Intensity trends (of 109 projects): 19 stable; 36 increasing (25 linear, 5 exponential,
  6 logarithmic); remainder decreasing/fluctuating.
- Contributor-count trends (of 94 projects): **83 fluctuating**, 4 decreasing, 7 increasing.

Frequent APIs (RQ4):
- Total APIs: Mockito **317** (only **109 / 34.5%** used), EasyMock **278** (**68 / 24.5%**),
  PowerMock **311** (**75 / 24.1%**).
- Mockito API-call distribution: stubbing **49%**, argument 20%, verification 16%,
  creation 14%, other 1%. EasyMock stubbing **50%**. PowerMock: 55% Mockito-related,
  34% EasyMock-related, 11% general.
- Top five APIs in each functional group cover **78% to 100%** of usage.

Informal mocking (RQ5):
- **143 / 193** projects contain "Customized Mock Classes"; **2,237** such classes;
  **9,697** test classes use them vs **28,792** test classes using framework APIs.
- On average **44%** of classes per project use Customized Mocks; across the 143 projects,
  **25%** of test classes use them.
- Random 2% sample = **44 classes**: 40 confirmed for test dependency isolation (all via
  inheritance), of which Xiao et al.'s refactoring tool applied to **20 (50%)**; 4 not
  for test dependency isolation.

Survey:
- **17 responses** from 300 invitations (**5.7%** response rate).
- Framework selection: Mockito/EasyMock/PowerMock chosen by **100% / 64.7% / 35.3%** of
  participants. **15 (88.2%)** report using a combination (vs 37% mined).
- Perceived Usage Intensity 18–76% (avg **55%**) and Mocking Intensity 9–75% (avg **44%**),
  vs mined averages of **10%** and **11%**. External vs internal mocking likelihood:
  63.4% vs 48.8%. **33%** confirmed using informal (non-framework) methods.

Comparison with Mostafa and Wang (2014):
- That prior study (5000 GitHub projects): **23%** used frameworks, **39.4%** of mocked
  objects for library classes. This study (Apache): **66%** and **61.1%** respectively.

## Scope, limitations, and gaps

- **Java only.** The JDT-based parser "is language-dependent, and cannot be used directly
  for another language, such as Python"; generalization to Python/JavaScript is explicitly
  out of scope.
- Test files identified by JUnit import — authors note this may misclassify integration
  tests or test-utility files (false positives).
- 16 projects excluded for compilation/config failures; results reflect the remaining 193.
- RQ3 evolution uses heuristics (developers on mock-using files = mock maintainers; one
  yearly snapshot) and the % -files-with-mocks metric can be skewed by refactoring.
- Survey response rate is low (5.7%) and likely biased toward intensive mockers; authors
  flag this discrepancy for SQ2 and SQ4 explicitly.
- Findings are descriptive of Apache (large, long-lived OSS); the authors caution against
  generalizing to projects "with completely different characteristics."

## Capture status

`transcript.md` is the **full open-access article** (curl, `ok`): abstract, all sections
(Introduction through Conclusion), RQ1–RQ5 results text, survey results, the Mostafa-and-Wang
comparison, limitations, and references. Figures and tables are present as image/"Full size
table" links only — numeric table cells embedded in prose are captured, but standalone
tables (e.g., Table 1 basic stats, Tables 6–10 API lists) are not rendered in the transcript.
