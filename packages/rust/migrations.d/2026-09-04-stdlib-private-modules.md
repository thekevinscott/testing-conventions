### Underscore-prefixed stdlib modules classify as stdlib

**Summary**

The stdlib set `unit lint` classifies Python imports against was an incomplete copy of
`sys.stdlib_module_names` — 217 of 300 names, missing every underscore-prefixed private module.
An unrecognized head is treated as a third-party package, so `__future__`, `_thread`, `_socket`,
`_ast` and the rest were flagged as un-mocked collaborators. The copy now carries the full set,
and keeps the names later Python versions dropped (`distutils`, `imp`, `smtpd`) so a consumer on
an older floor still reads them as stdlib.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

A colocated `*_test.py` that imports a private stdlib module now passes `unit lint`. The common
case is `from __future__ import annotations`, mandatory on a pre-3.10 floor, which previously
failed every test file carrying it. A `[[python.exempt]]` entry or a `gates` waiver taken to work
around that flag can be dropped.

**Verification**

Run the check over a unit suite whose tests open with the future import:

```sh
npx testing-conventions unit lint --language python src
```

The run prints nothing and exits 0, where it previously named the import:

```
src/widget_test.py:1: unmocked-collaborator — unit test imports `__future__` without mocking it — …
error: 1 isolation violation(s)
```
