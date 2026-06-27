# Summary

- **Title:** Large Language Models Cannot Self-Correct Reasoning Yet
- **Authors:** Jie Huang, Xinyun Chen, Swaroop Mishra, Huaixiu Steven Zheng, Adams Wei Yu,
  Xinying Song, Denny Zhou
- **URL:** https://arxiv.org/abs/2310.01798
- **Date:** Submitted 3 Oct 2023 (v1); last revised 14 Mar 2024 (v2)
- **Venue:** ICLR 2024 (per Comments field)
- **Source type:** (arXiv preprint; ICLR 2024 acceptance noted — abstract page only, full paper not captured)

## What the source claims

The paper critically examines *self-correction* in LLMs — particularly *intrinsic
self-correction*, where a model tries to revise its own answers without external feedback —
and finds it does not improve reasoning. From the abstract:

> "Central to our investigation is the notion of intrinsic self-correction, whereby an LLM
> attempts to correct its initial responses based solely on its inherent capabilities,
> without the crutch of external feedback."

The core finding:

> "In the context of reasoning, our research indicates that LLMs struggle to self-correct
> their responses without external feedback, and at times, their performance even degrades
> after self-correction."

The authors frame this as a corrective to optimistic claims about self-correction and "offer
suggestions for future research and practical applications in this field."

## Method / evidence type

Critical empirical examination of self-correction methods for LLM reasoning. The abstract
describes the conceptual framing (intrinsic self-correction without external feedback) and
reports qualitative directional findings (struggle to self-correct; performance can degrade).
No experimental details — models tested, benchmarks, or metrics — are given in the captured
abstract; these are in the full paper.

## Numbers recorded

The captured abstract contains **no numeric results** (no benchmarks, accuracies, or effect
sizes). No tables are present in the captured text.

## Scope, limitations, and gaps

- Scoped specifically to *reasoning* tasks and to *intrinsic* self-correction (no external
  feedback / oracle); the abstract's claim is bounded to that setting and does not speak to
  self-correction with external tools or verifiers.
- The captured abstract identifies findings directionally only — which models, which reasoning
  benchmarks, and the magnitude of any degradation are in the full paper, not captured.
- Results reflect a 2023-era assessment ("...Yet" in the title signals temporal scoping).
- Full paper body (PDF, experimental HTML, TeX source) is not captured.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
the title, authors, full abstract, submission history (v1, v2), and bibliographic metadata
(ICLR 2024; subjects cs.CL, cs.AI; arXiv DOI; CC BY 4.0 license). It does **not** contain the
full paper body — only the abstract and page chrome.
