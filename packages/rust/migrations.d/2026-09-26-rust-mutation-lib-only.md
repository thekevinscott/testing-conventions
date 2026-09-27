### Rust mutation judges the library unit target

**Summary**

Rust `unit mutation` now passes only `--cargo-test-arg --lib` to cargo-mutants, matching the library unit tier. Binary unit tests no longer judge library mutants.

**Required changes**

If a changed line survives and an integration or e2e test asserts it, add a `[[rust.exempt]]` entry for that exact line with `rules = ["mutation"]` and a `reason` naming the test and behavior. Add a library unit assertion for behavior owned by the library unit tier.

**Deprecations removed**

_None._

**Behavior changes without code changes**

Mutants killed only by binary unit tests now survive the library unit run. The gate reports unexempted survivors on changed lines.

**Verification**

Run `npx testing-conventions unit mutation --language rust <crate>` on a changed library line. A line asserted only by a binary unit test appears as a survivor; after a library unit assertion or a valid line-scoped exemption, the check passes.
