**Removed** **BREAKING** The Python package requires Python 3.11 or newer (`requires-python =
">=3.11"`). Python 3.9 and 3.10 are end of life, and GitHub's current Ubuntu runner image has no
3.9 build to test against. The package reads TOML with the stdlib `tomllib`. See
[`../migrations.d/2026-11-02-require-python-3-11.md`](../migrations.d/2026-11-02-require-python-3-11.md).
