# Summary

- **Title:** Automated Unit Test Improvement using Large Language Models at Meta
- **Authors:** Nadia Alshahwan, Jubin Chheda, Anastasia Finegenova, Beliz Gokkaya, Mark Harman, Inna Harper, Alexandru Marginean, Shubho Sengupta, Eddy Wang
- **URL:** https://arxiv.org/abs/2402.09171
- **Date:** Submitted on 14 Feb 2024 (v1)
- **Venue:** 32nd ACM Symposium on the Foundations of Software Engineering (FSE 24) (12 pages, 8 figures)
- **Source type:** (arXiv preprint; peer-reviewed conference paper noted — abstract page only, full paper not captured)

## What the source claims

Describes Meta's TestGen-LLM tool, which uses LLMs to automatically improve existing
human-written unit tests, gating generated test classes through filters that assure measurable
improvement and thereby eliminate hallucination-induced problems. Reports deployment results from
Meta test-a-thons for Instagram and Facebook.

Verbatim quotes from the abstract:

> "This paper describes Meta's TestGen-LLM tool, which uses LLMs to automatically improve existing
> human-written tests. TestGen-LLM verifies that its generated test classes successfully clear a
> set of filters that assure measurable improvement over the original test suite, thereby
> eliminating problems due to LLM hallucination."

> "We describe the deployment of TestGen-LLM at Meta test-a-thons for the Instagram and Facebook
> platforms."

> "In an evaluation on Reels and Stories products for Instagram, 75% of TestGen-LLM's test cases
> built correctly, 57% passed reliably, and 25% increased coverage."

> "During Meta's Instagram and Facebook test-a-thons, it improved 11.5% of all classes to which it
> was applied, with 73% of its recommendations being accepted for production deployment by Meta
> software engineers."

> "We believe this is the first report on industrial scale deployment of LLM-generated code backed
> by such assurances of code improvement."

## Method / evidence type

- Industrial deployment report / case study at Meta, using a filtered generate-and-test pipeline
  ("a set of filters that assure measurable improvement").
- Evaluation on Reels and Stories products for Instagram, plus Instagram and Facebook test-a-thons.
- Outcome metrics: build rate, reliable-pass rate, coverage increase, share of classes improved,
  and engineer acceptance rate.

## Numbers recorded

- Instagram (Reels and Stories) evaluation: "75% of TestGen-LLM's test cases built correctly, 57%
  passed reliably, and 25% increased coverage."
- Instagram and Facebook test-a-thons: "improved 11.5% of all classes to which it was applied,
  with 73% of its recommendations being accepted for production deployment by Meta software
  engineers."
- (Metadata only: "12 pages, 8 figures"; submission size 1,490 KB.)

## Scope, limitations, and gaps (as observable from the abstract)

- Single-organization deployment (Meta) on specific products (Instagram Reels/Stories, Facebook);
  generalization beyond this setting not addressed in the abstract.
- The abstract does not name which LLM(s) were used, nor the programming language(s) of the tests.
- Improvement is defined relative to existing human-written tests; the "filters" are referenced
  but not detailed in the abstract.
- No statistical-significance testing or variance reported in the captured text.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
title, authors, full abstract, and bibliographic metadata, but **not** the full paper body
(PDF at `/pdf/2402.09171` — not fetched; no experimental HTML link present on the page).
