**Fixed** The reusable workflow provisions node 24 in every job that invokes the CLI, so npm's
engine-aware resolution of the bare `testing-conventions` name returns the current release. Jobs
running on the runner's ambient node 22 resolved 0.0.86, the last release whose `engines.node` a
node 22 runner satisfies: `static` and `e2e-verify` on every language, and `unit-coverage` /
`coverage-changed` / `mutation` / `packaging` on any project that is not TypeScript. **BREAKING**
for a repo that passed against 0.0.86 — see `../migrations.d/2026-09-01-cli-node-engine-in-every-job.md`.
