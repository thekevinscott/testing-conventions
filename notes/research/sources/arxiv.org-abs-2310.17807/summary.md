# Summary

- **Title:** Clover: Closed-Loop Verifiable Code Generation
- **Authors:** Chuyue Sun, Ying Sheng, Oded Padon, Clark Barrett
- **URL:** https://arxiv.org/abs/2310.17807
- **Date:** Submitted 26 Oct 2023 (v1); last revised 16 Nov 2024 (v4)
- **Venue:** Not stated on the captured page (Comments: "add appendix")
- **Source type:** (arXiv preprint — abstract page only, full paper not captured)

## What the source claims

The paper introduces **Clover** ("Closed-Loop Verifiable Code Generation"), a paradigm that
uses *consistency checking* to filter out incorrect LLM-generated code. From the abstract:

> "Clover performs consistency checks among code, docstrings, and formal annotations. The
> checker is implemented using a novel integration of formal verification tools and large
> language models."

The authors provide "a theoretical analysis to support our thesis that Clover should be
effective at consistency checking," and evaluate empirically on a hand-built dataset:

> "We also empirically investigate its performance on a hand-designed dataset (CloverBench)
> featuring annotated Dafny programs at a textbook level of difficulty."

Reported experimental results (abstract):

> "(i) LLMs are reasonably successful at automatically generating formal specifications; and
> (ii) our consistency checker achieves a promising acceptance rate (up to 87%) for correct
> instances while maintaining zero tolerance for adversarial incorrect ones (no false
> positives). Clover also discovered 6 incorrect programs in the existing human-written
> dataset MBPP-DFY-50."

## Method / evidence type

A method paper combining (a) a theoretical analysis arguing Clover's consistency checking
should be effective, and (b) an empirical evaluation on a hand-designed benchmark
(CloverBench) of annotated Dafny programs, plus application to an existing dataset
(MBPP-DFY-50). The checker integrates formal verification tools with LLMs. Full method,
benchmark construction, and result breakdown are in the full paper, not the captured abstract.

## Numbers recorded

Exact figures from the captured abstract:
- Consistency checker acceptance rate for correct instances: "up to **87%**."
- Adversarial incorrect instances: "zero tolerance ... (**no false positives**)."
- "Clover also discovered **6** incorrect programs in the existing human-written dataset
  **MBPP-DFY-50**."

No other tables or per-condition figures are present in the captured text.

## Scope, limitations, and gaps

- Evaluated on a small, hand-designed benchmark (CloverBench) of "textbook level"
  Dafny programs; generalization to real-world, larger, or multi-language code is not
  addressed in the captured text. (Dafny is a verification-aware language; the approach
  depends on the availability of formal annotations and verification tooling.)
- "Up to 87%" is an upper-bound phrasing; the abstract does not report the distribution or
  the conditions under which 87% was achieved.
- The theoretical analysis supports that Clover "should be" effective — a conditional claim,
  not a guarantee on arbitrary inputs.
- Full paper body (PDF, experimental HTML, TeX source) is not captured.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
the title, authors, full abstract, submission history (v1–v4), and bibliographic metadata
(subjects cs.AI, cs.LG, cs.SE; arXiv DOI). It does **not** contain the full paper body — only
the abstract and page chrome.
