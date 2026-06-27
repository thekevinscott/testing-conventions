# Relevance — Harden and Catch for Just-in-Time Assured LLM-Based Software Testing

**Verdict:** relevant (broad-inclusion) — supplies the conceptual vocabulary and formal definitions (hardening vs catching tests, JiTTests, the Regression-Only Trap) for LLM-based test generation, framing what "good" tests for LLM-authored code should do.

## Salient sections

- Definition — "A hardening test is one that seeks to protect against future regressions, while a catching test is one that catches such a regression or a fault in new functionality introduced by a code change." → distinguishes two purposes a testing strategy must serve, with the claim that LLM test-generation has over-focused on hardening (the "Regression Only Trap").
- Coverage caveat — "It is well known, from many empirical studies, that increasing coverage does not provide a guarantee of increasing fault revelation [30, 33, 49, 52]." → cited (not newly measured) evidence cautioning against coverage as the optimization target, directly relevant to the goal's coverage-target dimension.
- Oracle value — "A relatively small amount of timely oracle information can dramatically enhance test effectiveness." → motivates oracle scavenging (LLMs reading PR titles/comments) as a strategy lever for catching tests.
- Deployment heuristic — proposed R@P with precision threshold p = 0.8 (false-positive:true-positive no worse than 1:4) → concrete acceptance criterion for an automated LLM test-catching gate.
- Industrial context — review of Meta's Sapienz, Fausta, TestGen-LLM, and the mutation-guided ACH tool (with ACH's six assurances) motivates the formalism but is experience report, not controlled experiment.

## Evidence weight

Peer-reviewed conference companion / keynote paper (FSE Companion '25; full PDF captured) that is definitional and analytical — non-empirical framework context; its empirical grounding is by reference to the authors' separately published systems (TestGen-LLM, ACH), and it reports no new benchmark/precision measurements of its own.
