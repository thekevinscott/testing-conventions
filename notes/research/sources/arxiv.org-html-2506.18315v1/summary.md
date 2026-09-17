# Summary

- **Title:** Use Property-Based Testing to Bridge LLM Code Generation and Validation
- **Authors:** Lehan He, Zeren Chen, Zhe Zhang, Jing Shao, Xiang Gao, Lu Sheng (Beihang University; Shanghai AI Laboratory; Shanghai Innovation Institute)
- **URL:** https://arxiv.org/html/2506.18315v1
- **Date:** Generated 23 Jun 2025 (LaTeXML timestamp on the HTML)
- **Venue:** Not stated on the captured HTML (arXiv preprint; code at github.com/HeLeHanPrivate/PBTwithCodeGen)
- **Source type:** (arXiv preprint; full paper HTML captured)

## What the source claims

Introduces **Property-Generated Solver (PGS)**, a framework that uses Property-Based
Testing (PBT) — validating high-level invariants instead of input-output examples — as
the core engine driving iterative LLM code generation. Two collaborating LLM agents (a
Generator and a Tester) decouple code generation from validation. Central claim: anchoring
refinement in verifiable properties beats conventional Test-Driven Development (TDD)
feedback, breaking the "cycle of self-deception" where generated tests share the code's
flaws.

Verbatim quotes:

> "Extensive experimental results on multiple code generation benchmarks demonstrate that
> Property-Generated Solver achieves substantial pass@1 improvements, ranging from 23.1% to
> 37.3% relative gains over established TDD methods."

> "On average, PGS achieves a substantial 9.2% absolute improvement in pass@1 scores over
> methods using prompting techniques."

> "On LiveCodeBench, using feedback solely from public test cases allows for the correction
> of 46.6% of initially flawed instances. Remarkably, when applying PBT-driven feedback to
> this same set of problems, the RSR within this subset further boosts to 75.9%."

> "\"Wrong Answer\" outcomes drop sharply from 25.3% (without refinement) to 10.5% with PGS.
> While \"Runtime Errors\" (including PBT assertion failures) increase from 4.6% to 11.8%,
> this reflects PGS converting latent logical flaws into explicit, actionable property
> violations, ultimately boosting \"Pass\" rates."

On feedback selection (delta-debugging-inspired):

> "selecting the failing input with the shortest length (\"Min Length\") yields the best
> pass@1 (74.5%), an improvement of +2.4% over median length and +3.0% over the longest
> inputs."

## Method / evidence type

- Controlled benchmark comparison of PGS against direct/CoT prompting and TDD/debugging
  baselines (Code-T, Self-Edit, Reflexion, MGDebugger, Self-Debugging, LDB).
- Benchmarks: **HumanEval** (164 problems), **MBPP** (~500 problems), **LiveCodeBench v5**
  (880 problems).
- Foundation models (weak→strong): DeepSeek-Coder-V2, Qwen2.5-Coder, DeepSeek-R1-Distilled-32B.
- Metrics: **pass@1** and **Repair Success Rate (RSR)**.
- Implementation details: temperature 0.5, max 32,768 tokens/call, refinement capped at
  **5 iterations** per problem, **6-second** per-test-case time limit, up to **5 properties**
  and **20 synthesized PBT inputs** per problem; feedback prioritizes the shortest
  violation-triggering input.
- Four research questions (RQ1 overall performance; RQ2 property-driven validation +
  feedback strategy; RQ3 viability of LLM-generated properties; RQ4 generalizability across
  difficulty/LLMs).

## Numbers recorded

Overall pass@1 / RSR (Table I), PGS row vs. baselines:

| Model | HumanEval pass@1 | HumanEval RSR | MBPP pass@1 | MBPP RSR |
|---|---|---|---|---|
| PGS — DeepSeek-Coder-V2 | 89.0 | 53.8 | 67.6 | 25.0 |
| PGS — Qwen2.5-Coder | 94.5 | 55.0 | 69.6 | 25.1 |
| PGS — DeepSeek-R1-Distilled-32B | 97.6 | 63.6 | 81.2 | 48.1 |

(Direct-prompting baselines for the same cells: HumanEval 76.2 / 87.8 / 93.3; MBPP 56.8 /
59.4 / 63.8.)

LiveCodeBench pass@1 "All" (Table II), PGS: DeepSeek-Coder-V2 **32.7**, Qwen2.5-Coder
**36.0**, DeepSeek-R1-Distilled-32B **74.5**. On **Hard** problems, PGS with
DeepSeek-R1-Distilled-32B reaches **40.7%** vs. direct prompting **28.1%**.

- Average absolute RSR improvement of ~**15.7%** over representative TDD baselines (HumanEval/MBPP).
- pass@1 gains over prompting range **4.2%** (Qwen2.5-Coder, LiveCodeBench) to **17.4%**
  (DeepSeek-R1-Distilled-32B, MBPP).
- Input-selection strategy (Table III, LiveCodeBench / DeepSeek-R1-Distilled-32B): Min
  Length 74.5% pass, 3.24k tokens; Min Runtime 73.3%, 3.28k tokens; Max Line Coverage 72.1%.
- Validation-generation accuracy vs. Direct Pass by difficulty (Table IV): Easy 82.4% vs.
  62.4%; Medium 62.8% vs. 17.5%; Hard 48.9% vs. 1.1%.

## Scope, limitations, and gaps

- Authors' stated threats: **data leakage** (benchmark samples may be in pretraining,
  possibly inflating absolute scores; argued not to affect relative comparisons);
  **generalizability** (only three benchmarks/three LLMs; multi-language untested); **PBT
  artifact quality** (trivial properties limit benefit); **hyperparameter** sensitivity.
- All benchmarks are Python function-level / competitive-programming tasks; no real-world
  repository evaluation.
- "Correctness" is judged solely by passing hidden benchmark test cases.
- Figures (x1–x6) and some exact per-cell values for Figures 5–6 are images not transcribed
  as text.

## Capture status

`transcript.md` is the full arXiv HTML (`capture_status: ok`, curl), including abstract,
all five sections, Tables I–V, threats to validity, related work, and 56 references.
Figures are referenced as images (`x1.png`–`x6.png`) and not captured as text; numeric
claims tied to Figures 5–6 are quoted from the surrounding prose.
