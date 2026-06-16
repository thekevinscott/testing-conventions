# Reusable-workflow self-test fixtures

Fixtures for [`../workflows/testing-conventions-selftest.yml`](../workflows/testing-conventions-selftest.yml),
which smoke-tests the reusable workflow
([`../workflows/testing-conventions.yml`](../workflows/testing-conventions.yml))
end to end.

- `clean/` — a fully-covered, colocated, mock-free Python suite. Every rule the
  reusable workflow runs over it (colocated-test, coverage, integration-lint)
  passes, so the `uses:` call must succeed.
- `below-floor/` — a Python suite whose coverage lands under the floor, so
  `unit coverage` exits non-zero — the build-failing behavior the workflow
  promises.

Each directory carries its own `testing-conventions.toml` with the
`[python].coverage` floor for that run. The self-test drives the *published*
`testing-conventions` binary (what consumers get via `npx`), so these fixtures
track the released surface rather than this branch's source.
