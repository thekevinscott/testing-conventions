# Relevance — Asleep at the Keyboard? Security of Copilot's Code

**Verdict:** relevant (broad-inclusion) — empirical evidence that LLM-generated code is frequently insecure, motivating *what* a testing strategy for LLM code must target (security-relevant defect classes), even though it studies code quality, not a testing method.

## Salient sections
- Headline finding (abstract, quoted in summary): "we produce 89 different scenarios for Copilot to complete, producing 1,689 programs. Of these, we found approximately 40% to be vulnerable." → Establishes a measured base rate of insecure LLM output, justifying security-oriented testing (e.g., CWE-targeted tests, static/dynamic security checks) as part of the strategy.
- Method tied to "MITRE's 'Top 25'" CWE list, examined across "diversity of weaknesses, diversity of prompts, and diversity of domains" (abstract). → Suggests which weakness categories a test suite for LLM code should prioritize.
- Premise that the model "will have learned from exploitable, buggy code" (abstract). → Frames why LLM output cannot be trusted unverified, supporting a verify-everything testing posture.

## Evidence weight
Primary empirical evidence about LLM-generated *code security* (peer-reviewed; IEEE S&P 2022), but it measures vulnerability rates, not the effectiveness of any testing technique — so it motivates testing goals rather than evidencing an optimal testing strategy. Only the abstract was captured; per-CWE breakdowns and the vulnerability-judging method are in the uncaptured full paper.
