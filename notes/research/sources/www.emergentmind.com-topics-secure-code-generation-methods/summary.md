# Summary

- **Title:** Secure Code Generation Methods
- **Authors:** Not attributed (Emergent Mind topic page; AI-generated overview, no named author)
- **URL:** https://www.emergentmind.com/topics/secure-code-generation-methods
- **Date:** "Updated 18 January 2026"
- **Venue:** Emergent Mind (commercial arXiv-aggregator website)
- **Source type:** (secondary/aggregator — AI-generated topic page; summarizes other work, NOT primary evidence)

> NOTE — This is a secondary aggregator page that synthesizes numbers from ~18 referenced
> papers. Every quantitative claim below is the aggregator's paraphrase of a cited primary
> study, not an original result. Attribute findings to the cited papers, and verify against
> the primaries before relying on any figure. Treat this page only as a map of the literature.

## What the source claims

A topic overview of methods for reducing vulnerabilities in LLM-generated code, organized
into seven families: (1) prompt engineering / interactive refinement, (2) training-time
alignment, (3) inference-time search and steering, (4) retrieval-augmented generation,
(5) post-processing / reflexion / critique loops, (6) limitations and adversarial
robustness, (7) domain-specific / lightweight architectures.

Verbatim framing quotes:

> "Secure code generation methods are systematic approaches that use prompt engineering,
> training alignment, and inference constraints to reduce vulnerabilities in LLM-generated
> code."

> "empirical studies indicate that naively generated code frequently embeds or propagates
> exploitable flaws, including a broad spectrum of Common Weakness Enumerations (CWEs)."

On limitations / adversarial robustness (the page's own caveat):

> "Adversarial perturbation of prompts—paraphrasing, cue inversion, or context manipulation—
> while preserving semantic intent, collapses the true secure-and-functional rate of leading
> defenses (SVEN, SafeCoder, PromSec) to 3–17%, irrespective of analyzer claims (Tessa et
> al., 11 Jan 2026)."

> "Static analyzer overestimation is profound: CodeQL 'secure' labels overstate true security
> by factors of 7–21, with up to 60% of 'secure' completions actually failing basic
> functionality unit tests."

> "several secure code generation techniques degrade the functional correctness of generated
> code in pursuit of security (often omitting required functionality entirely or emitting
> 'garbage' stubs)."

## Method / evidence type

- Not an empirical study. It is a curated/AI-generated literature digest citing 18 papers
  (mostly 2024–2026 arXiv preprints) with paraphrased headline numbers and a summary table.
- No methodology of its own; no data collection, no analysis. Each claim is sourced to a
  linked paper (e.g., `/papers/2407.07064`).

## Numbers recorded (all as reported by the aggregator, attributed to cited primaries)

- RCI prompting: "up to a 77.5% reduction in vulnerability density in GPT-4 relative to
  zero-shot prompting on LLMSecEval" (Tony et al., 2024).
- SecCode interactive encouragement prompting: ">76% vulnerability correction rate" after five
  iterations, ">89% after ten iterations" (Liu et al., 2024).
- SCGAgent: "25% increase in secured generations (FuncSec@1) over baseline prompts, with
  negligible (~2%) loss in functional correctness on CWEval C benchmarks" (Saul et al., 2025).
- PurpCode: "~9–20 point gains in secure-and-functional generation rates over base models"
  (Liu et al., 2025).
- Secure-Instruct: "14.3 point average absolute improvement ... (e.g., CodeLlama-7B from 47.6%
  to 69.8% secure)" (Li et al., 2025).
- LoRA/IA³ fine-tuning: "+5.4 to +6.9% Secure@1 improvement for C and C++ code LLMs"
  (Li et al., 2024).
- CodeBC: "57.5% reduction in generated vulnerability rate versus pre-trained CodeLlama"
  (Wang et al., 2025).
- Constrained decoding (CodeGuard+): "secure-pass@1 from 43.2% (vanilla sampling, CodeGen-2.7B)
  to 76.0% (+32.8 points)" (Fu et al., 2024).
- MoC steering: "raises security ratio by 8.9 points on Qwen2.5-Coder-7B" (Yu et al., 2025).
- RESCUE: "average +4.8 point SecurePass@1 improvement" (Shi et al., 2025).
- SOSecure: "71.7%–96.7% vulnerability fix rate (vs. 37.5%–56.5% without)" (Mukherjee et al.,
  2025).
- SecFSM: "security pass rate of 21/25 ... a 2–4× improvement" (Hu et al., 2025).
- Reflexion: "improves secure-generation accuracy from 70.74% (zero-shot) to 79.43% in three
  rounds" (Datta et al., 2025).
- GRASP: "Security Rates above 80% ... up to 88% relative improvement on zero-day
  vulnerabilities" (Patir et al., 2025).
- PSSec: "PowerShell repair success rates up to 87%" (Zhang et al., 2026).
- Summary table headline ranges: "RCI/EP/Reflexion 60–77% reduction in CWEs"; "Secure-Instruct
  +8–22 pp SecureRatio"; "Constrained Decoding +32.8 pp"; "MoC +8.9 pp"; "RAG +16–59 pp Fix
  Rate / +4.8 pp SecurePass@1"; "PurpCode +8.5–38 pp"; "PSSec 87% Fix Success."

## Scope, limitations, and gaps

- **Secondary, unverifiable in place:** the page does not reproduce any primary methodology;
  benchmarks, models, and metrics differ across the cited papers, so the numbers are not
  comparable to each other.
- **Authorship/provenance:** no named author; the site is an AI-driven arXiv aggregator
  ("chat with arXiv"). Reliability of paraphrase cannot be confirmed from this capture alone.
- **Recency / pre-print bias:** most references are 2024–2026 arXiv preprints; peer-review
  status varies and is not indicated per item.
- **Topic mismatch caution:** subject is *security* of LLM-generated code, not test-generation
  efficacy per se; relevance to testing conventions is via its joint security-/functionality
  evaluation and unit-test verification themes (e.g., SCGAgent's "LLM-generated unit test
  verification").

## Capture status

`transcript.md` is the rendered HTML topic page (`transport: curl`, `capture_status: ok`),
including the full prose, the summary table, and the 18-item reference list with arXiv IDs.
No primary paper text was fetched; all figures above are the aggregator's restatements.
