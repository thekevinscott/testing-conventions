### Pure value construction in unit tests

**Summary**

`unit lint` accepts exact named pure constructors and parsers used to prepare test values across Rust, Python, and TypeScript.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

Named pure operations no longer produce collaborator violations. Unlisted calls and imports, including effectful operations and glob imports, continue to produce violations.

**Verification**

Run the workflow's `unit-lint` gate for each package language.
