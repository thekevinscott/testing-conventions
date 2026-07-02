# Spikes

Time-boxed, throwaway prototypes that buy a decision with evidence. A spike
lives under `spike/` at the repo root, outside `packages/`, and follows spike
discipline: minimal runnable code, no coverage/mutation gates, no release
artifacts, local builds only. The deliverable is the *finding* — a filled
rubric and a recommendation — not the code; the code exists so the finding
can be re-verified after toolchains move.

Every spike directory carries:

- a `README.md` with exact build/run commands, and
- a `RESULTS.md` with the environment, the measured rubric, the
  recommendation, and a friction log (maturity blockers quoted verbatim).

## Current spikes

### `spike/` — agent-context delivery mechanism (issue #232)

Compares two ways to expose one Rust core (`print` / `write-block` / a
single-sourced `run(argv)` CLI) as CLI + in-process SDK across Python,
TypeScript, and Rust: **FFI** (pyo3 + napi-rs) vs **WASM component**
(wasm32-wasip2 via wasmtime-py + jco). Verdict and evidence:
`spike/RESULTS.md` — headline: FFI wins on per-call latency by ~60–100×;
WASM's warm in-process call is slower than the rejected subprocess baseline.
