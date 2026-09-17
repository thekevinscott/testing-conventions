### Python `unit coverage` no longer fails a branchless source

**Summary**

A Python package whose source has no branching constructs (`if`, loop, `try`, comprehension
guard) could not pass `unit coverage` at any coverage level, with any tests. `coverage.py` always
runs with `--branch`, so a report with zero measured branches can only mean the source genuinely
has none — but the floor read that zero as a misconfigured run and failed before checking any
percentage. The floor now treats a zero-branch report as vacuously at full branch coverage: the
percentage it checks is `coverage.py`'s own combined line + branch total, which already reduces to
the line percent when there is nothing to branch on.

**Required changes**

_None._ The change is inside the Python coverage run; a `uses:` call or direct
`testing-conventions unit coverage --language python <source>` needs no edit.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A Python package with no branching constructs and full line coverage now passes `unit coverage` at
the default config (`branch = true`, `fail_under = 100`) — previously it failed regardless of
coverage level or test count. A package that carried `coverage = { branch = false }` solely to work
around this failure can drop that override once its source stays branchless; `branch` still governs
whether the diff-scoped and line-exempt ratios fold branch arcs in alongside lines.

**Verification**

Run the gate on a Python package whose sources contain no branching constructs, with full line
coverage:

```console
$ testing-conventions unit coverage --language python src
```

The check passes without `[python] coverage.branch` set to `false`.
