# Relevance — Mutation-Guided LLM-based Test Generation at Meta

**Verdict:** relevant — empirical industrial deployment (Meta ACH, FSE'25 Industry Track) of a *mutation-guided* LLM test-generation strategy, reporting scale, an equivalent-mutant detector's precision/recall, and engineer-acceptance results — comparative-by-design evidence on how to gate and target test generation for LLM-authored/maintained code.

## Salient sections

- Targeted-mutation strategy: "ACH generates relatively few mutants ... compared to traditional mutation testing. Instead, it focuses on generating currently undetected faults that are specific to an issue of concern" and then "generates tests that can catch them, thereby 'killing' the mutants and consequently hardening the platform against regressions." → A concrete, deployed alternative to mass mutation: few concern-targeted mutants + tests that kill them. Bears directly on *what to test and at what granularity* — focus generation on undetected, high-concern faults rather than maximizing mutant count.
- Scale: "ACH was applied to 10,795 Android Kotlin classes in 7 software platforms deployed by Meta, from which it generated 9,095 mutants and 571 privacy-hardening test cases." → Large industrial footprint; the low mutant-to-class and test-to-mutant ratios quantify the "few, targeted" design in practice.
- Equivalent-mutant handling: LLM-based equivalent-mutant detection agent "achieves a precision of 0.79 and a recall of 0.47 (rising to 0.95 and 0.96 with simple pre-processing)." → Directly addresses mutation testing's classic equivalent-mutant cost (the same cost flagged in 2406.09843); shows it can be substantially mitigated, strengthening mutation as a practical adequacy gate.
- Human gate: "engineers accepted 73% of its tests, judging 36% to privacy relevant" and found non-targeted tests "useful for their other benefits." → Acceptance gate is human review; high acceptance indicates mutation-guided LLM tests reach production quality even when they miss the targeted concern.

## Evidence weight

- **Study type:** Industrial deployment report / case study; arXiv preprint submitted to FSE 2025 Industry Track.
- **Scope/sample:** Single organization (Meta); 10,795 Android Kotlin classes across 7 platforms; 9,095 mutants; 571 tests; Messenger/WhatsApp test-a-thon human evaluation.
- **Caveats:** Single org and single language/platform (Kotlin/Android); the claim that ACH "can harden code against any type of regression" is asserted from a privacy illustration, not demonstrated across concerns; underlying LLM(s) unnamed; the higher equivalent-mutant precision/recall depends on undescribed "simple pre-processing." Strong external validity at scale, limited generalizability.
- **Capture caveat:** Judgment rests on the captured abstract + metadata; full paper body not fetched.
