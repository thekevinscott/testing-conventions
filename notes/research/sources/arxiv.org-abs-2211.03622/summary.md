# Summary

- **Title:** Do Users Write More Insecure Code with AI Assistants?
- **Authors:** Neil Perry, Megha Srivastava, Deepak Kumar, Dan Boneh
- **URL:** https://arxiv.org/abs/2211.03622
- **Date:** Submitted 7 Nov 2022 (v1); last revised 18 Dec 2023 (v3)
- **Venue:** CCS '23: Proceedings of the 2023 ACM SIGSAC Conference on Computer and
  Communications Security, November 2023, Pages 2785-2799 (DOI 10.1145/3576915.3623157)
- **Source type:** (arXiv preprint of a peer-reviewed conference paper — abstract page only, full paper not captured)

## What the source claims

A user study of how people interact with an AI code assistant on security-related tasks. The
abstract describes it as:

> "the first large-scale user study examining how users interact with an AI Code assistant to
> solve a variety of security related tasks across different programming languages."

Principal findings stated in the abstract:

> "participants who had access to an AI assistant based on OpenAI's codex-davinci-002 model
> wrote significantly less secure code than those without access."

> "participants with access to an AI assistant were more likely to believe they wrote secure
> code than those without access to the AI assistant."

> "participants who trusted the AI less and engaged more with the language and format of their
> prompts (e.g. re-phrasing, adjusting temperature) provided code with fewer security
> vulnerabilities."

The authors also state they release their user interface "as an instrument to conduct similar
studies in the future" and provide an analysis of participants' language and interaction
behavior.

Note: the AI assistant studied is built on **OpenAI's codex-davinci-002** model (not an
Anthropic/Claude model).

## Method / evidence type

Controlled human-subjects user study (with-AI vs. without-AI access) across multiple
programming languages and several security-related tasks, plus self-reported perception data
and analysis of prompting/interaction behavior. The comment field notes statistical tests
and survey questions were added in revision ("update adds names of statistical tests and
survey questions, full version of conference paper"). Specific sample size, task list, and
test statistics are in the full paper, not the captured abstract.

## Numbers recorded

The captured abstract contains **no numeric results** (no sample size, effect sizes, or
p-values). The only quantitative metadata present:
- Comments: "16 pages, 16 figures"
- Journal reference page range: "Pages 2785-2799"

The abstract states the security difference was "significantly less secure" but gives no
figure.

## Scope, limitations, and gaps

- The AI assistant is based on a 2022-era model (codex-davinci-002); generalization to later
  assistants is not addressed in the captured text.
- The captured abstract reports directional findings only — no magnitudes, sample size, task
  composition, or which statistical tests were used.
- Self-reported security perception is a subjective measure; the abstract notes the
  perception gap but the captured text gives no breakdown.
- Full paper body (PDF, experimental HTML, TeX source) is not captured.

## Capture status

`transcript.md` is the arXiv abstract landing page (curl, `capture_status: ok`). It contains
the title, authors, full abstract, submission history (v1–v3), and bibliographic metadata
(CCS '23 journal reference, DOIs, subject cs.CR). It does **not** contain the full paper
body — only the abstract and page chrome.
