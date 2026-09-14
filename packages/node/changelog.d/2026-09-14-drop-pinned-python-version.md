**Fixed** The reusable workflow provisions the Python interpreter the package's own
`requires-python` declares, so a package with a higher floor (e.g. `>=3.14`) runs every
Python-environment gate. The `setup-uv` steps no longer pin `python-version: '3.12'`; `uv`
resolves an interpreter satisfying the manifest, downloading one if the runner has none. See
`../migrations.d/2026-09-14-drop-pinned-python-version.md`.
