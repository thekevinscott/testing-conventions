# Relevance — Revolutionizing software testing: Introducing LLM-powered bug catchers (Meta ACH)

**Verdict:** relevant (broad-inclusion) — vendor blog that frames mutation-guided, LLM-based test generation and points to the primary paper (arXiv 2501.12862); context for the "mutation testing" and "what gate decides a test is acceptable" axes of the strategy, not itself empirical evidence.

## Salient sections
- ACH definition (`summary.md` lines 13–22, Meta Engineering's words): "a system for mutation-guided, LLM-based test generation" that "targets *specific faults* rather than uncovered code, and generates tests it can *prove* will catch those faults." Bears on the goal's gate question — fault-detection over raw coverage as the acceptance criterion.
- Coverage critique (`summary.md` lines 28–34, quoting the post): "increasing coverage doesn't necessarily find faults"; ACH "targets specific faults, rather than uncovered code." Directly relevant to the goal's skepticism of coverage targets as the "optimal" criterion.
- Three-step workflow (`summary.md` lines 36–40): describe feared bugs → generate mutants → generate tests that kill them. Illustrates a mutation-guided strategy applicable to LLM-authored code.
- Novelty claim (`summary.md` lines 44–45): LLM-based test gen and LLM-based mutant gen "combined and deployed in large-scaled industrial systems" for the first time — situates this as the engineering framing of the primary study.
- Evidence caveat (`summary.md` lines 60–76): explicitly **a vendor announcement, not a study** — no experimental protocol, dataset, or numbers; the only effectiveness claim is self-reported ("engineers found ACH useful"). The substantiation lives in the uncaptured arXiv paper 2501.12862.

## Evidence weight
Vendor/industry engineering blog (Meta); non-empirical context — promotional summary of a primary peer-reviewed paper (arXiv 2501.12862) that is the actual evidentiary source and is not captured here.
