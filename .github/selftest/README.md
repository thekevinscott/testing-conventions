# Reusable-workflow self-test fixtures

Fixtures for [`../workflows/testing-conventions-selftest.yml`](../workflows/testing-conventions-selftest.yml),
which smoke-tests the reusable workflow
([`../workflows/testing-conventions.yml`](../workflows/testing-conventions.yml))
end to end.

- `clean/` — a fully-covered, colocated, mock-free Python suite. Every rule the
  reusable workflow runs over it (colocated-test, coverage, integration-lint)
  passes, so the `uses:` call must succeed. Also reused, under the zero-config
  default `["python", "typescript"]`, to prove that the source-free TypeScript
  jobs are skipped while the Python jobs still run (#94).
- `below-floor/` — a Python suite whose coverage lands under the floor, so
  `unit coverage` exits non-zero — the build-failing behavior the workflow
  promises.
- `no-sources/` — a directory with no language sources at all. Called with the
  zero-config default `["python", "typescript"]`, the workflow must skip every
  language's jobs and still pass (#94) — the dogfooding case of our own
  `packages/python`, which ships a wheel but carries no `.py`.

`clean/` and `below-floor/` each carry their own `testing-conventions.toml` with
the `[python].coverage` floor for that run. The self-test drives the *published*
`testing-conventions` binary (what consumers get via `npx`), so these fixtures
track the released surface rather than this branch's source.
