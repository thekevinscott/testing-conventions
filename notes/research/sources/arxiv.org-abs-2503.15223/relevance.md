# Relevance — Are "Solved Issues" in SWE-bench Really Solved Correctly? An Empirical Study

**Verdict:** relevant — empirical study quantifying how non-exhaustive test suites fail to validate LLM-generated patches, and demonstrating differential (behavioral) testing as a stronger acceptance gate.

## Salient sections

- "because testing is rarely exhaustive, a patch may pass the tests but nevertheless fail to match the developers' expectations" (abstract) → example-based pass/fail test suites are a weak acceptance gate for LLM-authored code; passing tests ≠ correct.
- "7.8% of all patches ... count as correct while failing the developer-written test suite" (abstract) → SWE-bench's own validation harness lets patches through that the human test suite rejects — evidence that the *gate definition* matters as much as the tests.
- "our novel automated technique reveals that even more (29.6%) plausible patches induce different behavior than the ground truth patches" via "PatchDiff ... differential patch testing, which automatically exposes behavioral discrepancies between two patches" → differential testing against a reference implementation catches divergences that the existing suite misses; a candidate strategy for LLM-authored code.
- "28.6% of behaviorally divergent patches are certainly incorrect" (manual inspection) and "inflation of reported resolution rates by 6.2 absolute percent points" → magnitude of the under-testing problem; quantifies the cost of relying on incomplete example-based suites.
- Behavioral differences "due to similar, but divergent implementations (46.8%)" and "patches that adapt more behavior than the ground truth (27.3%)" → characterizes the failure modes a testing strategy must catch (over-/under-scoped behavior).

## Evidence weight

- Study type: empirical study of plausible patches from three state-of-the-art issue-solving tools on SWE-bench Verified, combining an automated differential technique (PatchDiff) with manual inspection. arXiv preprint (v2, Sep 2025), related ACM DOI 10.1145/3744916.3764576 noted.
- Scope/sample: SWE-bench Verified only; three (unnamed) tools. Benchmark-bound; generalization to other benchmarks/tools not established in captured text.
- Caveats: "certainly incorrect" rate rests on human judgment (inter-rater reliability not stated in abstract). Full methodology and per-tool breakdowns are in the full paper; only the arXiv abstract page was captured, so all numbers here are from the abstract.
