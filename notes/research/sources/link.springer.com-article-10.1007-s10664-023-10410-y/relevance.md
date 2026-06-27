# Relevance — An empirical study on the usage of mocking frameworks in Apache software foundation (Xiao, Zhao, Wang, Li, Lim, Wei, Yu, Wang)

**Verdict:** relevant — peer-reviewed empirical repository-mining study (Empirical Software Engineering, 2024) quantifying *how mocking frameworks are actually used* across 193 Apache Java projects. It provides measured base rates and patterns for the mocking-practices category named in the rubric, on human-authored code.

## Salient sections

- **Mocking is used, but selectively (RQ1–RQ2).** 66% of projects use a mocking framework, yet "mocking is overall practiced quite selectively—not all test files use mocking, nor all dependencies of a test target are mocked." Concretely: in **70%** of projects <10% of test files mock; in **65%**, only 5–15% of dependencies are mocked; **58.5%** of mock-using files create ≤3 mocks. → Empirical baseline: even where mocking is available, restrained use is the norm — a counterweight to mock-everything LLM test generation.
- **Mocks target the external boundary (RQ2).** On average **61.1%** of mocked objects isolate *external library* dependencies (vs 39.4% in the earlier Mostafa & Wang 2014 GitHub study). → Reinforces the "mock the boundary, not domain logic" strategy with independent data.
- **A small API surface dominates (RQ4).** Of Mockito's 317 APIs only 34.5% are ever used; stubbing dominates call distribution (Mockito 49%, EasyMock 50%); the top five APIs per functional group cover **78–100%** of usage. → The effective mocking vocabulary is small — relevant to constraining/standardizing LLM-generated mock code.
- **"Informal" / customized mock classes are widespread (RQ5).** 143/193 projects contain hand-rolled "Customized Mock Classes" (2,237 of them; ~25% of test classes use them), which the authors call "potential sub-optimal mocking practices... as well as limitations of existing mocking frameworks." → Documents a recurring deviation from framework-idiomatic mocking.
- **Scale/recency drive adoption (RQ1).** 88% of large-scale vs 34% of small projects, and 73% of post-2010 vs 46% of pre-2005 projects, use a framework. → Context for when mocking strategy becomes load-bearing.
- **Survey vs. mined mismatch.** Developers self-report far higher mocking intensity (avg 44%) than mined reality (11%), and 88% claim multi-framework use vs 37% mined. → Practitioner perception overstates how much mocking actually happens — caution against opinion-driven mocking norms.

## Evidence weight

- **Study type:** Empirical repository mining (Eclipse JDT binding resolution; keyword + manual framework detection; pairwise correlation; yearly snapshots for evolution) triangulated with a developer survey.
- **Scope/sample:** 193 Apache Java projects (from 246, after exclusions), latest GitHub versions as of Oct 2022; survey n=17 (5.7% response).
- **Caveats / not on LLM code:** General SE study of **human-written Apache Java** code — descriptive of *usage*, not an effectiveness/fault-detection comparison and not on LLM-authored code. Java-only (JDT parser cannot handle Python/JS, explicitly out of scope); test-file detection by JUnit import may misclassify; RQ3 evolution uses heuristics; survey response rate low and biased toward intensive mockers; authors caution against generalizing beyond large, long-lived OSS.
