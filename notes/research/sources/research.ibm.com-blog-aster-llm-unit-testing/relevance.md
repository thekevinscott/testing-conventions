# Relevance — ASTER: Natural and multi-language unit test generation with LLMs

**Verdict:** relevant (broad-inclusion) — vendor research blog summarizing a primary ICSE 2025 study on LLM-generated unit tests; the most on-topic of this batch (LLMs producing tests, with coverage and preference comparisons), though the blog itself is promotional and the controlled results live in the uncaptured paper.

## Salient sections
- Approach (`summary.md` lines 13–24, IBM Research's words): "LLM prompting guided by lightweight program analysis can generate high-coverage and natural tests" for Java and Python, including mocking external dependencies. Describes an LLM-driven test-generation strategy and its acceptance criteria (compile + execute, coverage, naturalness).
- Coverage vs conventional tools (`summary.md` lines 26–34, cited figures): competitive with EvoSuite for Java SE ("slightly lower (-7%)" to "4x–5x higher"); for Java EE, exceeds EvoSuite by **+26.4% line / +10.6% branch / +18.5% method**; for Python, **+9.8% / +26.5% / +22.5%** vs CodaMosa. These are the comparative-effectiveness numbers the goal cares about — but they are self-reported via the blog, not verified here.
- Model-size effect (`summary.md` lines 36–37): smaller models lose only "0.1%, 6.3%, 2.7%" coverage vs larger — bears on cost/quality trade-offs in LLM test generation.
- Developer preference (`summary.md` lines 39–41): survey of "161 responses," "over 70% ... willing to add such tests with minor or no changes." A maintainability/acceptability signal, though the survey is internal to IBM.
- Failure modes named (`summary.md` lines 43–50): generated tests "lacking in 'naturalness'," "trivial or ineffective assertions," and that "models can hallucinate ... tests very often do not compile and run." Documents failure modes of LLM-generated tests — squarely within the goal's relevance rubric.
- Caveat (`summary.md` lines 72–82): vendor source, self-reported figures, internal survey; the controlled evidence is in the primary paper arXiv 2409.03093 (ICSE 2025, Distinguished Paper, SEIP track), not captured here.

## Evidence weight
Vendor/industry research blog (IBM); non-empirical context — promotional summary of a primary peer-reviewed paper (arXiv 2409.03093) that is the actual evidentiary basis and is not captured here.
