# Relevance — Non-determinism of ChatGPT in Code Generation

**Verdict:** relevant (broad-inclusion) — empirical evidence that LLM code generation is highly non-deterministic, motivating why LLM-authored code cannot be trusted from a single sample and why a testing strategy must account for output variance.

## Salient sections
- Core measurement (abstract, quoted in summary): across 829 problems, "the ratio of coding tasks with zero equal test output across different requests is 75.76%, 51.00%, and 47.56% for CodeContests, APPS, and HumanEval." → Same prompt yields functionally different code most of the time, so testing must be applied per-generation, not assumed stable across regenerations.
- Temperature finding: "setting the temperature to 0 does not guarantee determinism in code generation, although it indeed brings less non-determinism than the default." → Even decoding controls do not remove the need for verification; the gate must be the test, not the generation settings.
- Framing: "Non-determinism is a potential menace to scientific conclusion validity." → Cautions that benchmark/effectiveness claims about LLM code (and tests over it) must control for regeneration variance.

## Evidence weight
Primary empirical evidence about LLM *output variance* (measurement study, arXiv preprint with linked ACM journal DOI), but it measures non-determinism, not the effectiveness of any testing technique — so it motivates testing and flakiness-awareness rather than evidencing an optimal strategy. 2023-era ChatGPT snapshot; only the abstract was captured.
