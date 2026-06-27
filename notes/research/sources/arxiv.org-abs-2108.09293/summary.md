# Summary

- **Title:** Asleep at the Keyboard? Assessing the Security of GitHub Copilot's Code Contributions
- **Authors:** Hammond Pearce, Baleegh Ahmad, Benjamin Tan, Brendan Dolan-Gavitt, Ramesh Karri
- **URL:** https://arxiv.org/abs/2108.09293
- **Date:** Submitted 20 Aug 2021 (v1); last revised 16 Dec 2021 (v3)
- **Venue:** "Accepted for publication in IEEE Symposium on Security and Privacy 2022"
- **Source type:** (arXiv preprint; peer-reviewed conference acceptance noted — abstract page only, full paper not captured)

## What the source claims

The paper systematically investigates how often, and under what conditions, GitHub Copilot
(described as the first self-described "AI pair programmer," a language model trained on
open-source GitHub code) recommends insecure code. The central concern, from the abstract:

> "code often contains bugs - and so, given the vast quantity of unvetted code that Copilot
> has processed, it is certain that the language model will have learned from exploitable,
> buggy code. This raises concerns on the security of Copilot's code contributions."

Method described in the abstract: prompt Copilot to generate code in scenarios relevant to
high-risk CWEs (Common Weakness Enumerations), specifically those drawn from "MITRE's 'Top
25' list." Copilot's output is examined along three axes:

> "examining how it performs given diversity of weaknesses, diversity of prompts, and
> diversity of domains."

Headline finding (abstract):

> "In total, we produce 89 different scenarios for Copilot to complete, producing 1,689
> programs. Of these, we found approximately 40% to be vulnerable."

## Method / evidence type

Empirical security assessment of an AI code-generation tool (GitHub Copilot). Scenario-based
prompting tied to high-risk CWE categories (MITRE Top 25), evaluated across three dimensions
(weaknesses, prompts, domains). Vulnerability assessment of the generated programs. Full
methodological detail (how vulnerability was judged, tooling used) is in the full paper,
which is not captured here.

## Numbers recorded

Exact figures from the captured abstract:
- "89 different scenarios for Copilot to complete"
- "1,689 programs" produced in total
- "approximately 40% to be vulnerable"

No other tables or per-CWE/per-axis breakdowns are present in the captured text.

## Scope, limitations, and gaps

- Scoped to one tool (GitHub Copilot) at a 2021 snapshot; results are tied to that model
  version and may not generalize to later code assistants.
- "Approximately 40%" is an aggregate over scenarios deliberately chosen to be relevant to
  high-risk CWEs, not a random sample of programming tasks — so it characterizes behavior in
  security-sensitive contexts rather than overall code.
- The captured abstract gives no per-axis numbers, no definition of how "vulnerable" was
  determined, and no statistical/variance reporting; these are in the full paper.
- Full paper body (PDF at /pdf/2108.09293, TeX source at /src/2108.09293) is not captured.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
the title, authors, full abstract, submission history (v1–v3), and bibliographic metadata
(IEEE S&P 2022 acceptance; subjects cs.CR, cs.AI; DOI). It does **not** contain the full
paper body — only the abstract and page chrome.
