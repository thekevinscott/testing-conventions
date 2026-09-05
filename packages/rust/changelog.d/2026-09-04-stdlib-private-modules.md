**Fixed** Python `unit lint` reads the underscore-prefixed stdlib modules as stdlib. The embedded
copy of `sys.stdlib_module_names` held 217 of 300 names and dropped every private module, so
`from __future__ import annotations`, `import _thread`, and the rest fell through to the
third-party fallback and were flagged `unmocked-collaborator` with no patch that could satisfy
them. The copy is now complete. See
[`../migrations.d/2026-09-04-stdlib-private-modules.md`](../migrations.d/2026-09-04-stdlib-private-modules.md).
