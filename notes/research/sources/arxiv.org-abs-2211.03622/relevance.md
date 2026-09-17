# Relevance — Do Users Write More Insecure Code with AI Assistants?

**Verdict:** relevant (broad-inclusion) — empirical user-study evidence that AI assistance both lowers code security and inflates developers' confidence, motivating why a testing strategy for LLM code must impose independent verification rather than trust author/tool judgment.

## Salient sections
- Primary finding (abstract, quoted in summary): "participants who had access to an AI assistant ... wrote significantly less secure code than those without access." → Reinforces that LLM-involved code needs explicit security testing as part of the strategy.
- Overconfidence finding: "participants with access to an AI assistant were more likely to believe they wrote secure code than those without." → A perception gap argues the gate cannot be self-assessment; it must be an objective test/oracle, not author confidence.
- Behavioral finding: participants who "trusted the AI less and engaged more with the language and format of their prompts ... provided code with fewer security vulnerabilities." → Skeptical, iterative interaction correlates with fewer defects — weak support for adversarial/verify-heavy workflows.

## Evidence weight
Primary empirical evidence (controlled human-subjects study, peer-reviewed CCS '23), but it measures security outcomes and perceptions of human+AI coding, not the comparative effectiveness of testing techniques — so it motivates the need for testing, not which strategy is optimal. The studied assistant uses OpenAI codex-davinci-002 (2022-era); only the abstract was captured, with no magnitudes or sample size.
