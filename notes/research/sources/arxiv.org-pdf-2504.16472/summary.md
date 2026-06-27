# Summary

- **Title:** Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges
- **Authors:** Mark Harman, Peter O'Hearn, Shubho Sengupta (Meta Platforms; University College London; independent)
- **URL:** https://arxiv.org/pdf/2504.16472
- **Date:** arXiv v2, 14 May 2025 (cs.SE)
- **Venue:** FSE Companion '25 (33rd ACM International Conference on the Foundations of Software Engineering), Trondheim, Norway, June 23–28 2025 — accompanies the authors' keynote. DOI 10.1145/3696630.3734199
- **Source type:** (peer-reviewed conference companion / keynote paper; full paper PDF captured)

## What the source claims

A conceptual/position paper that formally defines **hardening** tests (protect against
future regressions) and **catching** tests (catch a regression or a fault in new
functionality), and introduces the **Catching Just-in-Time test (Catching JiTTest)
Challenge** — generating a test on the fly when a pull request is submitted, to catch a bug
before it lands in production. The authors argue regression-only test generation traps the
field in a "Regression Only Trap (ROT)," and that a solution to Catching JiTTest generation
can also be repurposed to find latent bugs in legacy code. They report initial results from
Meta's automated LLM-based hardening work.

Verbatim quotes:

> "A hardening test is one that seeks to protect against future regressions, while a catching
> test is one that catches such a regression or a fault in new functionality introduced by a
> code change."

> "We believe the Catching JiTTest Challenge to be the most challenging and impactful problem
> in software testing."

> "It is well known, from many empirical studies, that increasing coverage does not provide a
> guarantee of increasing fault revelation [30, 33, 49, 52]."

> "According to a recent study [42], the median response time for Git projects was estimated
> at 1.75 hours. ... a more realistic estimate of the typical industry standard time for the
> first human response to a pull request is closer to 8 hours."

> "\"A relatively small amount of timely oracle information can dramatically enhance test
> effectiveness.\""

## Method / evidence type

- **Formalisation**: a logic of revisions, builds/passes/fails predicates, and oracles
  (Definitions 1–15) covering timely tests, JiTTests, weak/strong hardening, weak/strong
  catching, regression catching, and functionality catching tests.
- **Case-by-case deployment analysis**: an eight-region Venn classification (Figure 1) and
  outcome tables (Tables 1–3) enumerating consequences of accepting/landing/discarding test
  signals; five of eight categories give misleading signals, three give true signals.
- **Industrial experience report**: review of Meta's deployed systems — Sapienz (since
  2017), Fausta (since 2021), TestGen-LLM, and the mutation-guided **ACH** tool — used to
  motivate the formalism. Not a controlled experiment; quantitative evaluation is minimal.
- **Four research challenges** posed (reduce mis-classification; distinguish weak vs. strong
  hardening; distinguish regression catching from false positives; the Catching JiTTest
  Challenge itself), plus an "oracle scavenging" approach using LLMs to read non-executable
  text (comments, PR titles).

## Numbers recorded

- Pull-request first-response time used to bound "real time": median **1.75 hours** (Git
  projects); realistic industry standard ~**8 hours** — framed as the compute budget for
  JiTTest generation.
- Proposed deployment goal **R@P = p** (recall at fixed precision), with a conservative
  precision threshold **p = 0.8**, i.e. false-positive-to-true-positive ratio no worse than
  **1:4** (engineers tolerate friction 20% of the time for 80% true catches).
- ACH gives **six** assurances (Buildable; Valid Regression Tests; Hardening; Additional
  Coverage; Relevant; Fashion Following); the first four are described as strict
  (verifiable) assurances, the last two as aspirational.
- **Eight** categories of test behaviour (combinations of strong/weak hardening on parent ×
  catching on child); JiTTest outcomes grouped into **five** categories (Tables 2–3).
- (No benchmark accuracy/precision measurements are reported — the paper is definitional and
  analytical, not an empirical evaluation.)

## Scope, limitations, and gaps

- This is explicitly an **open-research-challenges / keynote companion** paper: it
  formalises problems and proposes deployment heuristics rather than reporting measured
  outcomes of a new technique.
- Empirical grounding is by reference to the authors' separately published Meta systems
  (TestGen-LLM at FSE 2024, ACH at FSE 2025); this paper itself presents no new quantitative
  experiment.
- Oracle scavenging and Catching JiTTest generation are described as "currently open and
  untackled"; feasibility is argued via worked examples (e.g. the "TSIA/YOLO" parsimonious
  pull request), not measured.
- Definitions assume tests execute independently (a simplifying assumption the authors note
  can be relaxed).

## Capture status

`transcript.md` is the full PDF text (`capture_status: ok`, curl), including abstract,
Sections 1–7, all Definitions 1–15, Examples 1–6, Tables 1–3, Figure 1 (referenced), and
the reference list. Figures are referenced but rendered as linearized text; the traffic-light
colour coding of Tables 1–3 is described in a footnote but not visually preserved.
