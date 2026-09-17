# Relevance — Hermeticity | Bazel

**Verdict:** relevant (broad-inclusion) — defines hermeticity (isolation + reproducibility) as a precondition for trustworthy, deterministic test execution, providing terminology for the test-isolation/reproducibility dimension of a testing strategy for LLM-authored code.

## Salient sections

- Definition — "When given the same input source code and product configuration, a hermetic build system always returns the same output by isolating the build from changes to the host system." → the reproducibility property a test environment needs so that pass/fail signals are reliable, not host-dependent.
- Two aspects — "Isolation: Hermetic build systems treat tools as source code. They download copies of tools and manage their storage and use inside managed file trees." and "Source identity: ... ensure the sameness of inputs ... use this hash to identify changes." → concrete mechanisms (pinned tools, content hashing) for isolating tests from environmental drift.
- Troubleshooting — "Ensure null sequential builds ... compare a hash of the file contents and get results that differ, the build is not reproducible." and guidance on per-action sandboxing / building inside a Docker container with only the source tree → practical checks for detecting non-determinism that would make tests flaky.

## Evidence weight

Vendor conceptual documentation (Bazel docs; no study, data, or measurements) — non-empirical context only; it defines and prescribes hermeticity but provides no comparative or measured evidence, so it cannot ground an "optimal" claim.
