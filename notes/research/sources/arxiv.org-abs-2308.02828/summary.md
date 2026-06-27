# Summary

- **Title:** An Empirical Study of the Non-determinism of ChatGPT in Code Generation
- **Authors:** Shuyin Ouyang, Jie M. Zhang, Mark Harman, Meng Wang
- **URL:** https://arxiv.org/abs/2308.02828
- **Date:** Submitted 5 Aug 2023 (v1); last revised 17 Oct 2024 (v2)
- **Venue:** Related DOI 10.1145/3697010 (ACM journal; venue name not stated on the captured page)
- **Source type:** (arXiv preprint with linked ACM journal DOI — abstract page only, full paper not captured)

## What the source claims

The paper argues that LLM output for code generation is highly unstable across identical
prompts, and that this non-determinism threatens the validity of scientific conclusions drawn
from LLM code-generation research. From the abstract:

> "results from LLMs can be highly unstable; nondeterministically returning very different
> codes for the same prompt. Non-determinism is a potential menace to scientific conclusion
> validity."

ChatGPT was chosen "because it is already highly prevalent in the code generation research
literature." The empirical claim:

> "We report results from a study of 829 code generation problems from three code generation
> benchmarks (i.e., CodeContests, APPS, and HumanEval). Our results reveal high degrees of
> non-determinism: the ratio of coding tasks with zero equal test output across different
> requests is 75.76%, 51.00%, and 47.56% for CodeContests, APPS, and HumanEval, respectively."

On temperature:

> "setting the temperature to 0 does not guarantee determinism in code generation, although
> it indeed brings less non-determinism than the default configuration (temperature=1)."

Conclusion: "there is, currently, a significant threat to scientific conclusion validity,"
and researchers "need to take into account non-determinism in drawing their conclusions."

## Method / evidence type

Empirical measurement study. Repeated generation requests for the same prompts across three
established benchmarks, comparing output equality (test output equality) across requests, and
comparing temperature=0 vs. temperature=1 (default). Full protocol (number of repetitions,
exact equality metrics) is in the full paper, not the captured abstract.

## Numbers recorded

Exact figures from the captured abstract:
- "829 code generation problems" across three benchmarks (CodeContests, APPS, HumanEval).
- Ratio of coding tasks with zero equal test output across different requests:
  - CodeContests: **75.76%**
  - APPS: **51.00%**
  - HumanEval: **47.56%**
- Temperature finding: temperature=0 does not guarantee determinism, but yields "less
  non-determinism than the default configuration (temperature=1)" — no numeric value given in
  the abstract.

## Scope, limitations, and gaps

- Scoped to ChatGPT on three code benchmarks; generalization to other models is not addressed
  in the captured text.
- The captured abstract reports only the "zero equal test output" metric; other dimensions of
  non-determinism (e.g., syntactic similarity, pass-rate variance) and the exact request
  count per problem are in the full paper.
- Results are a 2023-era ChatGPT snapshot; model versions change over time.
- Full paper body (PDF, experimental HTML, TeX source) is not captured.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
the title, authors, full abstract, submission history (v1, v2), and bibliographic metadata
(subject cs.SE; arXiv DOI; related ACM DOI 10.1145/3697010; CC0 license). It does **not**
contain the full paper body — only the abstract and page chrome.
