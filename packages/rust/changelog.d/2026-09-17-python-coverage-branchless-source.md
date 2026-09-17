**Fixed** Python `unit coverage` no longer fails a source tree with no branching constructs. The
whole-tree floor read a `coverage.py` report with zero measured branches as a misconfigured run and
failed before checking any percentage — unreachable in practice, since the Python coverage run
always passes `--branch`, so a zero-branch report can only mean the source genuinely has no `if`,
loop, `try`, or comprehension guard. A report with no branches now reads as vacuously at full branch
coverage, matching how the TypeScript and Rust arms already treat an empty branch denominator. See
`../migrations.d/2026-09-17-python-coverage-branchless-source.md`.
