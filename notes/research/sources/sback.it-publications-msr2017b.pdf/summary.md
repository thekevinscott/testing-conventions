# Summary

- **Title:** To Mock or Not To Mock? An Empirical Study on Mocking Practices
- **Authors:** Davide Spadini, Maurício Aniche, Magiel Bruntink, Alberto Bacchelli
- **URL:** https://sback.it/publications/msr2017b.pdf
- **Date:** Not stated in captured body (references cite work up to Feb 2017)
- **Venue:** MSR 2017 (Mining Software Repositories) — per task framing; the venue/year is not printed in the captured text
- **Source type:** (peer-reviewed study; full paper PDF captured)

## What the source claims

Empirical study of how and why developers (do not) mock test dependencies, and what
challenges they face. Central finding: whether a dependency is mocked is highly dependent
on its architectural role/concern — dependencies that make testing difficult (databases,
web services, external resources) are frequently mocked, while classes encapsulating
domain concepts/rules are often not. A major reported challenge is keeping the mock's
behavior compatible with the original class.

Verbatim quotes:

> "The study reveals that the usage of mocks is highly dependent on the responsibility and
> the architectural concern of the class. Developers report to frequently mock dependencies
> that make testing difficult and prefer to not mock classes that encapsulate domain
> concepts/rules of the system."

> "Among the key challenges, developers report that maintaining the behavior of the mock
> compatible with the behavior of original class is hard and that mocking increases the
> coupling between the test and the production code."

> "RQ1 . Classes that deal with external resources, such as databases and web services are
> often mocked. Interestingly, there is no clear trend on mocking domain objects."

> "RQ2 . The architectural role of the class is not the only factor developers take into
> account when mocking. Respondents report to mock when to use the concrete implementation
> would be not simple, e.g., the class would be too slow or complex to set up."

> "RQ3 . The use of mocks poses several challenges. Among all, a major problem is
> maintaining the behavior of the mock compatible with the original class. Furthermore,
> mocks may hide important design problems."

> "Interestingly, a class being slow is not an important factor for developers when mocking."

On code metrics (coupling/complexity not predicting mock decisions):

> "We conjecture that the chosen code metrics are not enough to predict whether a class
> should be mocked."

## Method / evidence type

- Mixed quantitative + qualitative study of **four software systems**: three OSS (Sonarqube,
  Spring Framework, VRaptor) and one industrial Java system (Alura).
- Built a static-analysis tool, **MockExtractor**, to collect mocked/non-mocked dependencies
  from Mockito-based Java test suites.
- **Manual analysis** of a random sample of dependencies (95% confidence level, 5% error)
  to classify architectural concern; categories grouped via card sort.
- **3 interviews** (semi-structured, via Skype) with developers from three of the projects
  (no Sonarqube developer available).
- **Survey** of 105 professionals (5-point Likert; English + Brazilian Portuguese).
- A validation interview with a main developer of Mockito (referred to as D4).
- An initial quantitative replication mining mocking practices (Wilcoxon rank sum test,
  Cliff's delta).

## Numbers recorded

Sample / corpus:

- "we manually analyze how more than 2,000 test dependencies are treated."
- "These results are supported by a structured survey with more than 100 professionals."
- Table I totals (N=4 systems): 13,892 classes; 1,818k LOC; 4,419 test units; 1,122 test
  units with mock; 2,568 mocked dependencies; 35,745 not mocked; sample sizes 844 mocked +
  1,334 not mocked = **2,178** analyzed.
- "we analyzed 4,419 test units of which 1,122 (25.39%) contain at least one mock object."
- "From the 38,313 collected dependencies from all test units, 35,745 (93.29%) are not
  mocked while 2,568 (6.71%) are mocked."
- Unique dependencies: "11,824 not mocked and 938 mocked"; "650 dependencies (70% of all
  dependencies mocked at least once) were both mocked and not mocked in the test suite."
- Categorization: 116 categories reduced to **7**; "The final agreement on the 7 categories
  was 89%."

How often each role is mocked (Figure 2, N = 2,178), mocked vs non-mocked:

| Role | % mocked (count) | % non-mocked (count) |
|---|---|---|
| Database | 72% (167) | 28% (64) |
| Web service | 69% (182) | 31% (82) |
| External dependencies | 68% (140) | 32% (67) |
| Domain object | 36% (319) | 64% (579) |
| Java libraries | 7% (12) | 93% (160) |
| Test support | 6% | 94% (358) |

Quantitative replication:

- "the mean coupling of mocked classes is 5.89 with a maximum of 51, while the mean coupling
  of non mocked classes is even slightly higher (7.131) with a maximum of 187 ... the overall
  difference is negligible (Wilcoxon p-value<0.001, Cliff's Delta=−0.121)."
- "The mean complexity of mocked classes is 10.58 with a maximum of 89.00, while the mean
  complexity of non mocked classes is 16.42 (max 420). Difference is also negligible
  (Wilcoxon p-value=5.945e−07 , Cliff's delta=−0.166)."
- Database dependencies mocked outside their own test: "In case of Alura, we found that 90%
  of database dependencies are mocked when not in their specific test unit. When extending
  this result to all the projects, we obtain an average of 81%."
- "over ~38,000 analyzed dependencies the unit under test is never mocked in any of the
  projects."

Survey responses (N = 105):

- "48% of respondents said they always or almost always mock classes that are highly
  coupled, and 45.5% when the class difficult to set up."
- "(50.4% and 34.5% of respondents affirm to never or almost never mock [slow / complex]"
- Native libraries: "82% of them affirm to never or almost never mock such dependencies."
- Respondents "always or almost always mock Web services (~82%), External dependencies
  (~79%) and Databases (~71%)."
- Demographics: "21% of the respondents have between 1 and 5 years of experience, 64%
  between 6 and 15 and 15% have more than 15 years of experience." Most-used languages:
  Java 24%, JavaScript 19%, C# 18%. Most-used frameworks: Mockito 33%, Moq 19%, Powermock 5%.

Cited prior work (Mostafa et al. [32]): "23% of the projects are using at least one mocking
framework and that Mockito is the most widely used (70%)."

## Scope, limitations, and gaps

Stated by the authors (Threats to Validity, Section IV-C):

- **Construct:** MockExtractor is static and "is not able to capture dynamic behavior"
  (e.g., mocks created in helper classes); such cases would be counted "non mocked".
  Mitigated by large random samples and manual inspection of 100 test units.
- **Internal:** each class analyzed by a single researcher (cross-validated 100 instances,
  89% agreement); interviewee opinions may be subject to social-desirability bias.
- **External:** "Our sample contains four Java systems (one of them closed source), which is
  small compared to the overall population of software systems that make use of mocking ...
  Further research in different projects in different programming languages should be
  conducted." Survey may suffer self-selection bias; response rate not calculable.
- Code metrics (CBO, McCabe) did not separate mocked from non-mocked classes — authors say
  metrics are "not enough to predict whether a class should be mocked."
- Single mocking framework focus (Mockito); spies explicitly out of scope.

## Capture status

`transcript.md` is the full paper PDF re-extracted to text (`capture_status: ok`). It
contains the abstract, full body (Introduction through Conclusion), all tables, figure
captions, and references. Figures themselves are rendered only as garbled text/point
clouds; numeric values cited above were taken from the in-text prose and table cells that
survived extraction.
