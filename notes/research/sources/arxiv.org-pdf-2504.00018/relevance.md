# Relevance — SandboxEval: Towards Securing Test Environment for Untrusted Code

**Verdict:** relevant (broad-inclusion) — empirical work on safely executing untrusted LLM-generated code, plus a measured failure rate for LLM-generated test cases, both of which constrain a testing strategy for LLM-authored code (where to run tests, and whether LLMs can be trusted to write the tests).

## Salient sections

- Abstract/§ method — "SandboxEval tests 51 properties associated with malicious and potentially harmful code execution scenarios, such as sensitive information exposure, filesystem manipulation, and external communication." → defines a concrete, hand-crafted suite for validating that the environment running LLM code is safe; the "where to execute tests" precondition of a testing strategy.
- LLM test-generation finding — "We found that LLM-generated test cases often contain unusable code with outcomes that were difficult to assess" and "on average, only 16.47% of the LLM-generated code was syntactically valid based on the test descriptions used as queries" (Python variant highest at 27.8%, rising to ~40.13% after stripping non-code) → directly documents a failure mode of LLM-authored tests, evidence that LLM-generated tests need validation/filtering before use.
- Case study (Table IV) — running the hand-crafted suite in a real Dyff instance on Kubernetes/gVisor with deny-all NetworkPolicy left External Communications and Dangerous Operations all "Denied" while several Expose-System probes were "Accessed" → empirical demonstration that sandbox configuration is partial and must be measured, not assumed.
- Related-work figures — ~40% of Copilot programs vulnerable (Pearce et al.); 32.8% of snippets with security issues (Fu et al.) → quantifies why LLM-authored code warrants security-focused testing.

## Evidence weight

Empirical artifact-construction + deployment case study (arXiv preprint, full PDF captured; one platform, Python-only, suite explicitly non-exhaustive) — primary empirical evidence for the test-execution-safety and LLM-test-validity sub-questions; not a comparative study of test-design techniques, so adjacent to the core "which strategy detects most faults" question.
