# Relevance — Benchmarking Reward Hack Detection in Code Environments via Contrastive Analysis (TRACE)

**Verdict:** relevant (broad-inclusion) — empirical benchmark on detecting reward hacks in code environments, which speaks to the gameability of automated checks/tests used as the acceptance gate for LLM-authored code.

## Salient sections

- Abstract — "a novel taxonomy of reward exploits spanning across 54 categories and ... TRACE ... a synthetically curated and human-verified benchmark containing 517 testing trajectories." → catalogs the ways code can satisfy a check without meeting intent, i.e. the threat model a test/gate strategy must defend against.
- Abstract result — "models capture reward hacks more effectively in contrastive settings than in isolated classification settings, with GPT-5.2 ... achieving the best detection rate at 63%, up from 45% in isolated settings." → measured evidence that how you frame the detection task (contrastive vs isolated) materially changes catch rate; informs how to structure checks that catch gaming.
- Abstract — "state-of-the-art models struggle significantly more with semantically contextualized reward hacks compared to syntactically contextualized ones." → semantic gaming is the harder case for automated detection, a limit on test/oracle-based gating of LLM code.

## Evidence weight

Empirical benchmark study (taxonomy + human-verified benchmark + detection-rate and ablation experiments; arXiv abstract only, Jan 2026, no stated venue) — primary empirical evidence, but on reward-hack detection (gameable-check robustness) rather than directly comparing test techniques; topically adjacent to the testing-strategy goal.
