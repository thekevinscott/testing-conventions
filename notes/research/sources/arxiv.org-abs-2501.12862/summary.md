# Summary

- **Title:** Mutation-Guided LLM-based Test Generation at Meta
- **Authors:** Christopher Foster, Abhishek Gulati, Mark Harman, Inna Harper, Ke Mao, Jillian Ritchey, Hervé Robert, Shubho Sengupta
- **URL:** https://arxiv.org/abs/2501.12862
- **Date:** Submitted on 22 Jan 2025 (v1)
- **Venue:** Submitted to FSE 2025 Industry Track
- **Source type:** (arXiv preprint — abstract page only, full paper not captured)

## What the source claims

Describes Meta's ACH system for mutation-guided LLM-based test generation. Rather than generating
many mutants, ACH generates relatively few mutants targeting a specific concern (e.g. privacy),
then generates tests that "kill" those undetected faults to harden the platform against regressions.
Reports an industrial application across thousands of Android Kotlin classes, plus an LLM-based
equivalent-mutant detector and engineer acceptance results from test-a-thons.

Verbatim quotes from the abstract:

> "This paper describes Meta's ACH system for mutation-guided LLM-based test generation. ACH
> generates relatively few mutants (aka simulated faults), compared to traditional mutation
> testing. Instead, it focuses on generating currently undetected faults that are specific to an
> issue of concern."

> "From these currently uncaught faults, ACH generates tests that can catch them, thereby `killing'
> the mutants and consequently hardening the platform against regressions."

> "We use privacy concerns to illustrate our approach, but ACH can harden code against {\em any}
> type of regression."

> "In total, ACH was applied to 10,795 Android Kotlin classes in 7 software platforms deployed by
> Meta, from which it generated 9,095 mutants and 571 privacy-hardening test cases."

> "ACH also deploys an LLM-based equivalent mutant detection agent that achieves a precision of 0.79
> and a recall of 0.47 (rising to 0.95 and 0.96 with simple pre-processing)."

> "ACH was used by Messenger and WhatsApp test-a-thons where engineers accepted 73% of its tests,
> judging 36% to privacy relevant."

> "We conclude that ACH hardens code against specific concerns and that, even when its tests do not
> directly tackle the specific concern, engineers find them useful for their other benefits."

## Method / evidence type

- Industrial deployment report / case study at Meta of the ACH system.
- Application scope: "10,795 Android Kotlin classes in 7 software platforms deployed by Meta."
- Includes an LLM-based equivalent-mutant detection agent evaluated by precision/recall.
- Human evaluation via Messenger and WhatsApp test-a-thons (acceptance and privacy-relevance
  judgments by engineers).

## Numbers recorded

- "10,795 Android Kotlin classes in 7 software platforms"; "9,095 mutants and 571 privacy-hardening
  test cases."
- Equivalent-mutant detection agent: "precision of 0.79 and a recall of 0.47 (rising to 0.95 and
  0.96 with simple pre-processing)."
- Test-a-thons: "engineers accepted 73% of its tests, judging 36% to privacy relevant."
- (Metadata only: v1 submission size 2,715 KB.)

## Scope, limitations, and gaps (as observable from the abstract)

- Single-organization deployment (Meta) on Android Kotlin classes; generalization to other
  languages/platforms not addressed in the abstract.
- Privacy is used as the illustrative concern; the claim that ACH "can harden code against any type
  of regression" is asserted, not demonstrated across concerns in the abstract.
- The abstract does not name the underlying LLM(s).
- Equivalent-mutant detection's higher precision/recall figures depend on "simple pre-processing,"
  which is not detailed in the captured text.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
title, authors, full abstract, and bibliographic metadata, but **not** the full paper body
(PDF at `/pdf/2501.12862`, HTML at `/html/2501.12862v1` — not fetched).
