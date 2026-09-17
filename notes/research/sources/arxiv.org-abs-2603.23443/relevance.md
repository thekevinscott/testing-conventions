# Relevance — Evaluating LLM-Based Test Generation Under Software Evolution

**Verdict:** relevant — large-scale empirical measurement of how LLM-generated tests behave under code change, documenting that they rely on surface cues and lose regression awareness as programs evolve.

## Salient sections

- "Under SACs, the pass rate of newly generated tests drops to 66%, and branch coverage declines to 60%" and "More than 99% of failing SAC tests pass on the original program while executing the modified region, indicating residual alignment with the original behavior rather than adaptation to updated semantics" → LLM-generated tests fail to catch semantic-altering changes; a testing strategy for LLM code cannot assume the generated tests track behavior changes.
- "Performance also declines under SPCs despite unchanged functionality: pass rates fall to 79% and branch coverage to 69%" + "Models generate more new tests while discarding many baseline tests, suggesting sensitivity to lexical changes rather than true semantic impact" → instability under semantics-preserving refactors; LLM test suites are brittle to lexical/syntactic change, undermining regression value.
- "LLMs achieve strong baseline results, reaching 79% line coverage and 76% branch coverage with fully passing test suites on the original programs" → high coverage on the original code coexists with poor adaptation under evolution — reinforces that coverage on a static snapshot overstates real test quality (echoes the coverage-is-weak theme).
- "current LLM-based test generation relies heavily on surface-level cues and struggles to maintain regression awareness as programs evolve" → core failure mode an optimal strategy must compensate for (e.g., mutation/behavioral gates, regenerating and re-validating tests after changes).

## Evidence weight

- Study type: large-scale empirical study using an automated mutation-driven framework across 8 LLMs and 22,374 program variants, with two change classes (semantic-altering SAC, semantic-preserving SPC). arXiv preprint (v1, Mar 2026); no peer-reviewed venue stated.
- Scope/sample: broad on variants/models, but the eight models and the programming language(s) are not named in the abstract; variants are mutation-generated, so correspondence to real developer code evolution is not quantified.
- Caveats: metrics reported are pass rate and coverage; fault-detection (mutant-kill) outcomes are not stated in the abstract. Only the arXiv abstract page was captured.
