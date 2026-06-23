# Dagger migration — handoff

**Status:** Exploratory. No commitment, no work started. This records the case and a
plan so it can be picked up cold.

## Why this exists

Closing [#230](https://github.com/thekevinscott/testing-conventions/issues/230) (the
"red attestation" gate) surfaced a more general limit: the deterministic checks we want
for a **dark-factory model** — agents producing code behind machine-enforced gates, with
no human on the line — can't be expressed in GitHub Actions. The blocker was never runner
speed; it's that the GitHub Actions / Checks **data model** treats results as coarse
per-job pass/fail and exposes no first-class per-test aggregation or cross-commit result
history. The #230 *forcing* half (prove a prior commit was validly red, survive squash)
needed an authenticated per-SHA `check-runs` read — a mechanism class unlike every other
rule here (static checks + pure git).

The checks we want and GitHub can't express deterministically:

- Gate on **specific actions / computed conditions**, not just job exit codes.
- Emit **rich, machine-readable feedback** an agent consumes to self-correct (typed
  results, not a red X + logs).
- **Dynamically fan out / in** based on what a run discovers.
- Drive **custom UIs** for factory oversight.

Either we drop these, or we adopt a programmable CI. This documents the second path.

## Why Dagger

Dagger is a programmable automation engine (pipelines as real code: Go / Python / TS),
container-based, runs locally / in CI / standalone. It now ships a native `LLM` primitive
and positions explicitly as the runtime for *agentic software factories* — which is our
use case. Against the four criteria:

| Criterion | In Dagger |
|---|---|
| Gate on specific actions | Gates are ordinary code — gate on anything you can compute. |
| Rich machine feedback | Functions return **typed values**, not logs+exit codes — the right substrate for an agent. |
| Dynamic fan-out / in | Just programming (loops, conditionals over discovered state). |
| Custom UIs | Build your own on the engine's GraphQL API / Dagger Cloud traces. |

Alternatives considered and why not (for *this*): the HN "drop-in GitHub CI replacements"
(Blacksmith, Depot, Namespace, WarpBuild, RunsOn, Ubicloud, …) are **runner swaps** that
keep the Actions model byte-for-byte — zero help. Buildkite is the strongest buy-and-bend
(dynamic pipelines, metadata, annotations) but you never fully own the UI. Tekton + Chains
is the build-your-own option with **SLSA provenance attestation built in** — the closest
off-the-shelf cousin of the attestation idea — but a heavy k8s commitment. Earthly tried
to win on CI speed and shut down (July 2025); Dagger absorbed its users. Dagger is the
center of gravity for an agent-driven, programmable line.

## What moves vs. what stays

Dagger replaces the **orchestration**, not the checks.

- **Stays:** the Rust binary is still the source of truth; every rule is still
  `testing-conventions <rule>` run in a container. Python/Node wrappers unchanged.
- **Moves:** `.github/workflows/testing-conventions.yml` (the ~513-line reusable workflow)
  and `.github/actions/detect/detect.py` (the language fan-out) become Dagger Functions.

## Migration plan (phased, incremental)

### Phase 1 — wrap, don't rewrite (~days, zero behavior change)

- `dagger init` a module. **SDK language: open question** (see below) — TS or Python keeps
  it closest to the existing wrappers; Go is also supported.
- One Function per rule: `colocatedTest`, `colocatedTestChanged`, `unitLint`,
  `unitCoverage`, `coverageChanged`, `mutation`, `integrationLint`, `e2eVerify`,
  `packaging`. Each is a container that installs the toolchain and runs the binary.
- A top-level `check(source)` Function reproduces `detect.py`'s fan-out as a **loop in
  code** instead of a matrix + `GITHUB_OUTPUT` plumbing.
- The GitHub workflow collapses to one step: `dagger call check --source=.`. Triggers and
  runners stay GitHub's; the Dagger Engine bootstraps per-run.
- **Win immediately:** orchestration is typed, unit-testable, and runs identically on a
  laptop (`dagger call ...`). This phase is reversible and can sit behind the existing
  workflow while we dogfood it.

### Phase 2 — the gates GitHub couldn't express

- Gates now return typed values, so the #230 red-loop reconciliation becomes a Function:
  run the integration suite, parse the per-test report **in-process** (no JUnit-artifact
  upload dance), compare against the declared expected-fail list, return a structured
  verdict the agent reads directly.
- Add the other dark-factory gates here as plain code.

### Phase 3 — leave GitHub's orchestration (optional; the real dark factory)

- Run the Dagger Engine on our own infra (or Dagger Cloud), drive `dagger call` from our
  own webhook / control plane, build the oversight UI on its GraphQL API.
- This is the **only** part that's a true migration *off* GitHub. Everything before it
  still runs inside GitHub Actions.

## Two load-bearing facts

- **Distribution transfers cleanly.** Dagger modules are callable by Git ref —
  `dagger call -m github.com/thekevinscott/testing-conventions@v0 check` — the native
  analog of a reusable workflow. Consumers invoke it from *their* CI, even GitHub Actions.
  So "ship a drop-in" survives the move; the product could become a Dagger module
  alongside (or instead of) the reusable workflow.
- **The honest gap.** Dagger caches operations (content-addressed) but has **no built-in
  queryable per-commit result history**. The #230 *forcing* half (read a prior commit's
  results, squash-proof) still needs our own datastore or Dagger Cloud. Programmability
  removes the *fight*, not the *work*.

## Open questions to resolve before starting

1. **SDK language** for the module (TS / Python / Go) — and how it interacts with the
   "least parity" rule, since it adds a language to the stack.
2. **Where the Engine runs** in steady state (per-run in GHA vs. self-hosted vs. Dagger
   Cloud) — gates Phase 3.
3. **Product question:** do we ship a Dagger module as the drop-in, keep the reusable
   workflow, or both? Affects whether external consumers are asked to adopt Dagger.
4. **Cross-commit state store** for the forcing-style gates (own datastore vs. Dagger
   Cloud) — the gap above.
5. **Dogfooding:** how this repo runs its own checks through the module during Phase 1
   without regressing the current green CI.

## References

- [Agents in your Software Factory — the LLM primitive in Dagger](https://dagger.io/blog/llm/)
- [Self-healing pipelines with AI agents — Dagger](https://dagger.io/blog/automate-your-ci-fixes-self-healing-pipelines-with-ai-agents/)
- [dagger/dagger (GitHub)](https://github.com/dagger/dagger)
- [A soft landing for Earthly users — Dagger](https://dagger.io/blog/earthly-to-dagger-migration)
- [#230 — Red attestation (closed, not planned)](https://github.com/thekevinscott/testing-conventions/issues/230)
