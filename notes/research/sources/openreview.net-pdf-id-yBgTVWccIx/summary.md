# Summary

- **Title:** DafnyBench: A Benchmark for Formal Software Verification
- **Authors:** Chloe Loughridge, Qinyi Sun, Seth Ahrenbach, Federico Cassano, Chuyue Sun, Ying Sheng, Anish Mudide, Md Rakib Hossain Misu, Nada Amin, Max Tegmark
- **URL:** https://openreview.net/pdf?id=yBgTVWccIx
- **Date:** Published in Transactions on Machine Learning Research (01/2025)
- **Venue:** Transactions on Machine Learning Research (TMLR), reviewed on OpenReview
- **Source type:** Full peer-reviewed paper (PDF captured)

## What the source claims

DafnyBench is presented as the largest benchmark for ML-based formal software verification:
a suite of ~750+ Dafny programs on which LLMs are tested for their ability to auto-generate
the annotations (loop invariants, assert statements) the Dafny verifier needs to prove a
program meets its specification. Central empirical claims: the best model/prompt reaches a
68% success rate, retrying with error-message feedback helps only modestly, and success
falls as program size and required-annotation quantity grow.

Verbatim quotes:

> "We test the ability of LLMs such as GPT-4 and Claude 3 to auto-generate enough
> annotations for the Dafny formal verification engine to successfully verify over 750
> programs with about 53,000 lines of code. The best model and prompting scheme achieved
> 68% success rate, and we quantify how this rate improves when retrying with error message
> feedback and how it deteriorates with the amount of required code and annotations."

> "Table 2 shows that Claude 3 Opus performed best, achieving a success rate ∼ 68%."

> "We see that the best models succeeded on the first try about 54%, with rapidly
> diminishing returns after that, approaching a plateau about 65% for n ∼ 5. This suggests
> that the LLMs are not great at taking Dafny error messages into consideration, or struggle
> to cope with the underlying task."

> "Figure 5a show that the success rate drops with program size... Figure 5b shows that the
> success rate drops with the annotation quantity, defined as the number of characters in
> the lines of compiler annotations."

> "we deem a benchmark program 'solved' if a model can make it pass the Dafny verifier
> without modifying the requires and ensures statements in the program and without using
> {:verify false} or assume false"

## Method / evidence type

- Benchmark construction: **782** standalone compiling Dafny ground-truth programs from
  three sources — GitHub scrape (556), Clover (62), Dafny-synthesis (164). Dafny 4.3.0;
  GitHub set deduplicated (~15,000 .dfy files → ~5,000) via a minhash/LSH algorithm.
- Task = **fill_annotations**: all assert/invariant statements removed (without markers);
  the LLM must restore annotations so the program verifies.
- Evaluation metric (success): verified by Dafny AND preserves all requires/ensures AND no
  `{:verify false}` / `assume false` cheating.
- Models: GPT-4o, GPT-4 Turbo, GPT-3.5 Turbo, Claude 3 Opus, CodeLlama-7b-Instruct-hf.
- Hyperparameters: max_tokens = 4096, temperature = 0.3, up to **n = 10** attempts per file
  (early-stop on success; failed attempts fed the Dafny error message back to the model).
- Failure analysis: failures categorized into **nine** types (verification logic error, code
  logic error, type error, resolution error, syntax issue, altered specification, timeout,
  trivial verification, other).

## Numbers recorded

Benchmark: **782** programs, ~**53,000** lines of code, "over 750 programs." Programs average
**2.04** methods, **0.98** functions, **1.31** lemmas. Programs verifying with no annotations:
GitHub 113/556, Clover 23/62, Dafny-synthesis 72/164. Prior benchmarks: Clover **66**,
Dafny-synthesis **153** (66 + 153 = **219** total).

Success rates (Table 2, n = 10 attempts):
| Model | % Success |
|---|---|
| No LLM (baseline) | 26.9 |
| GPT-3.5 Turbo | 44.0 ± 1.8 |
| GPT-4 Turbo | 59.8 ± 1.8 |
| GPT-4o | 59.3 ± 1.8 |
| Claude 3 Opus | 67.8 ± 1.7 |
| CodeLlama-7b-Instruct-hf | 28.0 ± 1.6 |

- Best model (Claude 3 Opus): **~68%** (67.8 ± 1.7).
- First-try success ~**54%**; plateau ~**65%** by n ≈ 5.
- Dafny verifies some programs even without annotations — the **26.9%** "No LLM" baseline.
- Comparison datasets (Table 1): math theorem proving 15,000–138,000 proofs; program
  synthesis 165–10,000+ programs; formal verification 66 (Clover) and 153 (Dafny-synthesis).

## Scope, limitations, and gaps

- **Data contamination** is flagged as a significant limitation: GitHub-scraped programs may
  overlap with model training data, possibly inflating scores.
- The benchmark does **not** assess translating natural language into formal specifications
  (only filling annotations given the spec); authors call this a key untested skill.
- Programs each fit in a **single file**; multi-file/external-library Dafny is excluded.
- Does not implement Clover's full six-way consistency check; "solved" = passes verifier
  without spec edits or cheating.
- Prompts were "manually but not very systematically tuned"; fixed temperature and 4096-token
  cap; no fine-tuned/custom-trained models provided — authors note room for improvement.

## Capture status

`transcript.md` is the **full paper PDF** (curl, `ok`): abstract, introduction, related work
(Table 1), construction (sources, fill_annotations task), experiments (Table 2, attempts
plot, size/annotation-quantity findings, nine failure types), discussion/conclusions, full
references, and appendices A–F (prompts, minhash pseudocode, scraped-repo license tables,
Dafny verification examples). Figures (distributions, success-vs-attempts, failure counts)
are referenced but their images are not rendered; their textual findings are captured.
