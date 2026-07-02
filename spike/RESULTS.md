# RESULTS — agent-context spike: FFI vs WASM

## Environment (results are version-sensitive)

- Linux 6.18.5, x86_64, glibc (Debian-based container). Docker CLI present but no daemon → the musl/Alpine probes could not be executed here.
- rustc/cargo 1.94.1; targets: x86_64-unknown-linux-gnu, wasm32-wasip2
- Python 3.11.15 (venv); Python 3.13.12 (regular GIL build) for the abi3 probe; no free-threaded 3.13t available
- Node v22.22.2, npm 10.9.7
- pyo3 0.29.0, maturin 1.14.1; napi 3.10.0, napi-derive 3.5.8, @napi-rs/cli 3.7.2
- wit-bindgen 0.58.0, wasmtime-py 46.0.1, jco 1.24.6 (preview2-shim 0.19.0), esbuild 0.28.1
- Bench tree: 30 markdown files, 233,060 bytes. Warm = median of 1000 in-process calls; cold = median of 20 fresh processes.

## The rubric

| Axis (→ thesis) | FFI | WASM | pure-Py baseline | subprocess baseline |
|---|---|---|---|---|
| Per-call latency, warm (→2) | Py **0.185 ms** / Node **0.334 ms** / Rust direct 0.150 ms | Py **18.4 ms** / Node **22.2 ms** | 0.814 ms | Py 2.63 ms / Node 3.92 ms |
| Per-call latency, cold/first (→2) | Py 13.0 ms / Node 32.5 ms (interpreter floors: 10.6 / 32.1 ms — the binding adds ~2.4 / ~0.3 ms) | Py 262 ms / Node 253 ms (in-process re-instantiate: 174 ms; jco cannot re-instantiate in-process at all — module singleton) | n/a | 2.63 ms |
| Lines of hand-written glue per language (→1) | Py 55 (lib.rs) · Node 41 (lib.rs 34 + bin.js 4 + build.rs 3) · Rust 0 | guest 34 + WIT 17 (shared) · Py 78 (sdk 68 + cli 10, all hand-written — no bindgen anymore, see below) · Node 16 (sdk 12 + cli 4) | — | — |
| Business logic duplicated? (→1) | none — bindings call `core` | none — guest calls `core`; core byte-identical between tracks | — | — |
| Output byte-identical across all 3 langs? (→3) | yes — sha256 `a93cba0f3359` everywhere | yes — same sha as FFI and native | — | — |
| API-surface parity / asymmetries (→3) | pyo3 panics → catchable `PanicException`; **napi panics abort the host** (below). Result objects: attrs (Py class) vs plain object (Node) — cosmetic | err arm of `result<T,E>` **returned as a bare value, not raised/thrown** in wasmtime-py (caller must type-sniff); jco throws `ComponentError`. Trap **poisons the instance in wasmtime-py only** | — | — |
| Build-matrix complexity (→4) | one wheel per platform (abi3 collapses Python versions); one `.node` npm package per platform; crates.io = source | **one `.wasm` for every platform/language**; native burden lives in the runtimes (`wasmtime` pip dep, jco's preview2-shim) | — | — |
| Artifact size(s) (→4) | wheel 492 KB (cp39-abi3, manylinux_2_34); `.node` 1,014 KB; native bin 936 KB | component 507 KB; jco dist 868 KB + preview2-shim dep 415 KB; Python ships the 507 KB wasm + `wasmtime` dep (~30 MB installed) | — | — |
| Registry artifacts actually built (→4) | wheel ✔ (maturin); napi package built for linux-x64-gnu ✔ (per-platform npm split not exercised — needs publish) | none published; wheel/npm for WASM would be platform-*independent* packages | — | — |
| Crash isolation: host survives a fault? | panic: Py **yes**, Node **no** (SIGABRT); segfault: both hosts die (exit 139) | **yes, always** — panic/abort → catchable trap; "segfault" (null write) isn't even a fault in linear memory. jco instance stays usable; wasmtime-py instance is poisoned and must be re-instantiated (174 ms) | — | — |
| Bundler-compatible (esbuild)? | fails by default; **works with `--loader:.node=copy`** (`--loader:.node=file` builds but breaks at runtime) | **no** — bundles, then hangs forever at runtime (preview2-shim worker thread + relative `.core.wasm` asset). Expected "fine", observed broken | — | — |
| Filesystem model & per-call glue | none — host fs directly | 2–8 lines once per process (code below); **jco's default preopen is the entire host `/`** — sandboxing is opt-in | — | — |
| Toolchain maturity blockers (quote them) | none hit; napi panic-abort is the sharp edge | `wasmtime.bindgen` **removed** ("`No module named wasmtime.bindgen`"); replaced by an untyped dynamic API. Positive: stable rustc emits components directly (`cargo build --target wasm32-wasip2`) — cargo-component not needed | — | — |

## Probe evidence (the receipts)

### FFI hazards

1. **esbuild:** default → `✘ [ERROR] No loader is configured for ".node" files`.
   `--loader:.node=file` bundles but fails at runtime (`TypeError: print is not a function` — the loader substitutes a path string for the module). `--loader:.node=copy` works end to end.
2. **Crash isolation:** pyo3 converts a panic to `pyo3_runtime.PanicException` — host survives, instance fine. napi-rs 3.10 does **not** catch it:
   `fatal runtime error: failed to initiate panic, error 5, aborting` → SIGABRT, Node exits 134. A real segfault (null write) kills both hosts (exit 139), as expected.
3. **abi3 + 3.13:** the cp39-abi3 wheel installs and runs unmodified on CPython 3.13.12 (GIL build). Free-threaded 3.13t is not installable in this environment — untested; the known state stands (abi3 wheels don't load on free-threaded builds until PEP 803 abi3t exists).
4. **musl/Alpine:** not executable here (no Docker daemon). Known answer unchallenged: glibc wheel/`.node` need separate musl builds; this is FFI's per-platform tax.
5. **npm optional-deps:** not exercised — requires actually publishing the per-platform package matrix. The napi tooling generates it; npm's optional-deps bugs (npm/cli#4828, #8320) remain a real-world risk we did not reproduce.

### WASM hazards

1. **FS-capability glue** (the actual setup code, once per process):

   Node (jco):
   ```js
   import { _setPreopens } from '@bytecodealliance/preview2-shim/filesystem';
   import { tool } from './dist/agent_context_wasm.js';
   _setPreopens({ '/work': hostDir });          // DEFAULT is { '/': '/' } — whole host fs!
   const r = tool.print(['**/*.md'], '/work');
   ```
   Python (wasmtime-py):
   ```python
   linker = Linker(engine); linker.add_wasip2()
   wasi = WasiConfig(); wasi.preopen_dir(host_dir, "/work"); wasi.inherit_stdout()
   store.set_wasi(wasi)
   instance = linker.instantiate(store, component)   # then 2-level export lookup per function
   ```
2. **Cold vs warm:** warm 18–22 ms per call; wasmtime-py in-process re-instantiation 174 ms; fresh-process cold ~253–262 ms. jco has **no** in-process re-instantiation path (transpiled module instantiates at import, singleton) — after a trap you keep the (luckily unpoisoned) instance or restart the process.
3. **End to end:** yes — `**` glob (readdir), UTF-8 read, and the `write-block` file write all work through preopens in both hosts, byte-identical to native.
4. **Maturity blockers:**
   - `python -m wasmtime.bindgen` → `No module named wasmtime.bindgen` (wasmtime-py 46 removed the generated-bindings path the handoff and older docs assume). The replacement dynamic API is workable but untyped: exports are looked up by string in two levels, `result<T, E>`'s **err arm is returned as a plain value** (indistinguishable by type from an ok `string`), and records come back as opaque attribute objects.
   - After any trap, wasmtime-py's instance is dead: `wasm trap: cannot enter component instance`.
   - Counter-finding: no `cargo-component` needed — stable rustc's `wasm32-wasip2` target + wit-bindgen 0.58 emitted a working component first try.
5. **esbuild:** bundles without error, then **hangs forever at runtime** (timeout kill; zero output). Two causes: the transpiled module `fs.readFile`s `*.core.wasm` relative to its own URL (fixable by copying the asset), and preview2-shim's io layer spawns a `worker_threads` Worker whose URL doesn't survive bundling (not fixable with a flag). The handoff's "expected: fine" is **wrong** for the Node/jco path.
6. **Artifacts:** one 507 KB component; per-platform native burden confirmed to live in the runtimes (`wasmtime` wheel, Node's own engine), not in our package.

## Recommendation: FFI (pyo3 + napi-rs)

**Thesis 2 decides it, and it isn't close.** Warm SDK calls: FFI 0.185/0.334 ms vs WASM 18.4/22.2 ms — a ~60–100× gap. The damning comparison: WASM *in-process* is ~7× slower than the **subprocess baseline we rejected for being too slow** (2.6–3.9 ms), and 22× slower than naive pure Python. If subprocess spawn overhead disqualifies a mechanism, WASM-through-preopens disqualifies itself twice over. Cold is worse: ~a quarter second per fresh process vs ~milliseconds over the interpreter floor for FFI.

- **Thesis 1 (no reimplementation):** tie. Both tracks kept the core byte-identical with under ~100 lines of glue per language; wit-bindgen's generated glue is nicer in principle, but wasmtime-py's bindgen removal means the Python side is hand-written string-keyed lookups — *more* fragile than the pyo3 macros.
- **Thesis 3 (parity):** FFI, narrowly. Output is byte-identical on every surface in both tracks. But WASM's host-side semantics diverge (err arm: bare return in Python vs thrown in Node; trap poisoning in Python only), while FFI's one asymmetry — napi aborting on panic — is fixable at our binding layer (wrap exports in `catch_unwind` / keep panics out of the contract).
- **Thesis 4 (registry presence):** WASM, genuinely. One platform-independent artifact vs a per-platform matrix is a real operational win, and its sandbox/crash story is strictly better. It loses anyway because theses are ANDed and 2 fails.

**The single finding that would flip the call:** the per-call cost. If the WASI-preopen filesystem path (or Component-Model hosts generally) got within ~2× of native — or if `agent-context`'s real call pattern turns out to be one call per process, where 250 ms cold dominates everything and FFI's cold advantage shrinks to interpreter noise — the one-artifact, sandboxed, crash-isolated WASM story wins thesis 4 and neutralizes 2. Re-run `bench/` on each major wasmtime/jco release; the harness is one command per language.

Ship note for the FFI track: adopt the ruff/uv precedent the prior art predicted — abi3 wheels + napi per-platform packages + the native binary for the CLI, and add an explicit `catch_unwind` in the napi glue so a core panic surfaces as a JS error instead of killing the consumer's process.

## What fought you (friction log)

- **napi panic abort** (Track A): the expectation that napi-rs converts panics to JS exceptions is stale for napi 3.10 defaults — `failed to initiate panic, error 5` and the host is gone. Cost an hour of "is this our bug"; it's not: the fix belongs in the binding glue.
- **wasmtime-py bindgen is gone** (Track B): every doc trail (and §5 of the handoff) points at `python -m wasmtime.bindgen`; wasmtime-py 46 has no such module. The dynamic replacement works but you discover its semantics (bare err returns, Record-as-attrs, two-level export lookup, instance poisoning) by experiment, not docs.
- **jco bundling hang** (Track B): the worst kind of failure — esbuild reports success, the bundle then blocks forever with no output. Root causes (relative `.core.wasm` read + worker-thread URL) are only findable by grepping the transpiled output.
- **jco's default preopen is `/`** (Track B): surprising for a "sandboxed" mechanism — full host filesystem unless you call `_setPreopens` (an underscore-private API) yourself.
- **What didn't fight:** stable rustc → component with zero extra tooling; maturin/abi3 (one command, wheel runs on 3.9→3.13); glob/read/write through preopens worked first try in both hosts; byte-identical output across all five surfaces required no effort at all — the shared core did exactly what thesis 1 hoped.
