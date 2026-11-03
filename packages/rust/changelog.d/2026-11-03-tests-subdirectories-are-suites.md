**Changed** `integration lint` accepts any directory under `tests/` as a suite. `tests/perf/`,
`tests/smoke/`, and the like are left unscanned; only `tests/integration/` and `tests/e2e/` are
held to the integration rules. `unknown-tier` now flags only a test file directly in `tests/`.
