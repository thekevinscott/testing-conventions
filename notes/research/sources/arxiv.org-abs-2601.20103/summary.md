# Summary

- **Title:** Benchmarking Reward Hack Detection in Code Environments via Contrastive Analysis
- **Authors:** Darshan Deshpande, Anand Kannappan, Rebecca Qian
- **URL:** https://arxiv.org/abs/2601.20103
- **Date:** Submitted 27 Jan 2026 (v1)
- **Venue:** No venue stated. Comment lists a dataset: https://huggingface.co/datasets/PatronusAI/trace-dataset. Subjects: cs.SE; cs.AI; cs.LG.
- **Source type:** (arXiv preprint — abstract page only, full paper not captured)

## What the source claims

Introduces **TRACE** (Testing Reward Anomalies in Code Environments), a benchmark for
reward-hack detection, plus a taxonomy of reward exploits. Central claim: LLM evaluators
detect reward hacks better in a contrastive anomaly-detection setup than in isolated
classification, and they struggle more with semantically (vs syntactically) contextualized
hacks.

Verbatim quotes from the abstract:

> "we propose a novel taxonomy of reward exploits spanning across 54 categories and introduce
> TRACE (Testing Reward Anomalies in Code Environments), a synthetically curated and
> human-verified benchmark containing 517 testing trajectories."

> "Unlike prior work that evaluates reward hack detection in isolated classification
> scenarios, we contrast these evaluations with a more realistic, contrastive anomaly
> detection setup on TRACE."

> "Our experiments reveal that models capture reward hacks more effectively in contrastive
> settings than in isolated classification settings, with GPT-5.2 with highest reasoning mode
> achieving the best detection rate at 63%, up from 45% in isolated settings on TRACE."

> "state-of-the-art models struggle significantly more with semantically contextualized
> reward hacks compared to syntactically contextualized ones."

> "ablation studies showing that the ratio of benign to hacked trajectories and analysis
> cluster sizes substantially impact detection performance."

## Method / evidence type

- New taxonomy (54 categories) + new benchmark (TRACE).
- Benchmark is "synthetically curated and human-verified."
- Comparison of two evaluation setups: isolated classification vs contrastive anomaly
  detection.
- Qualitative analyses of model behaviors + ablation studies (benign:hacked ratio, cluster
  size).
- Benchmark and evaluation harness released.

## Numbers recorded (verbatim from abstract)

- "54 categories" (taxonomy of reward exploits).
- "517 testing trajectories" (benchmark size).
- "GPT-5.2 with highest reasoning mode achieving the best detection rate at 63%, up from 45%
  in isolated settings on TRACE."

## Scope, limitations, and gaps (as observable from the abstract)

- Benchmark is "synthetically curated" — abstract does not state how closely synthetic
  trajectories match real RL training environments.
- Best result is one model (GPT-5.2, highest reasoning mode) at 63%; full per-model results
  not in abstract.
- "semantically vs syntactically contextualized" distinction is asserted; supporting numbers
  not in captured text.
- Full methodology, model list, and ablation magnitudes are in the full paper, **not
  captured here** — only the arXiv abstract page was fetched.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
title, authors, full abstract, and bibliographic metadata, but **not** the full paper body
(PDF at `/pdf/2601.20103`, HTML at `/html/2601.20103v1` — not fetched). Paper is dated
January 2026.
