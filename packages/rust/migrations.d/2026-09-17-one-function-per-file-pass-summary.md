### A passing `one-function-per-file` run reports what it scanned

**Summary**

A passing `unit one-function-per-file` run emitted only the version banner, so a real pass over a
populated tree and a vacuous pass over an empty or wrongly-scoped directory produced identical
output and exit code. The run now prints a one-line summary to stderr — `one-function-per-file:
scanned N file(s), 0 violations` — so the file count distinguishes a real scan from a no-op.

**Required changes**

_None._ The change adds a line of output; no flag, config key, or exit code moved.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A passing run's stderr now carries a second line after the version banner. A consumer that
asserted a clean run's stderr held nothing but the banner — for example `stderr == "testing-conventions
<version>\n"` — needs to allow for the new summary line.

**Verification**

```console
$ testing-conventions unit one-function-per-file --language typescript src
testing-conventions <version>
one-function-per-file: scanned 8 file(s), 0 violations
```

An empty or wrongly-scoped directory now reports `scanned 0 file(s)` instead of the same silent
banner-only output as a real pass.
