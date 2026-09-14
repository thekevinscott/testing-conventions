**Fixed** Python `unit coverage` scopes the denominator to the scanned path's sources, so an
editable path dependency's source tree no longer counts against the package's own floor. A
package whose own tree was fully covered failed on code that belongs to a different package — one
reached through an editable install (`[tool.uv.sources] a = { path = "../a", editable = true }`),
whose `.pth` resolves to a tree outside `site-packages` that coverage.py's default "measure
everything not stdlib or third-party" picked up. The run now passes `--source`, matching the
TypeScript arm, which already scopes the denominator to the scanned path. See
`../migrations.d/2026-09-14-python-coverage-scope-editable-deps.md`.
