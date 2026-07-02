# agent-context spike — FFI vs WASM

Throwaway prototypes for issue #232's delivery-mechanism decision: expose one
Rust core (`print` + `write-block` + a single-sourced `run(argv)` CLI) as a
CLI **and** an in-process SDK in Python, TypeScript, and Rust, via two
mechanisms:

- **Track A — FFI** (`ffi/`): pyo3 + maturin wheel; napi-rs `.node` addon.
- **Track B — WASM** (`wasm/`): one `wasm32-wasip2` component, consumed by
  `wasmtime-py` (Python) and `jco` (Node).

`core/` is byte-identical between tracks. Findings live in
[`RESULTS.md`](./RESULTS.md). Spike discipline applies throughout: minimal,
runnable, no production gates.

## Prerequisites

```
rustup target add wasm32-wasip2
python3 -m venv .venv && .venv/bin/pip install maturin wasmtime
npm i -g @napi-rs/cli @bytecodealliance/jco esbuild
```

## Build everything

```
cargo build --release -p agent-context-core            # native CLI (both tracks' baseline)
cd ffi/py   && VIRTUAL_ENV=$PWD/../../.venv ../../.venv/bin/maturin develop --release && cd ../..
cd ffi/node && napi build --platform --release && cd ../..
cargo build --release -p agent-context-wasm --target wasm32-wasip2
cd wasm/node && npm i && jco transpile ../../target/wasm32-wasip2/release/agent_context_wasm.wasm -o dist && cd ../..
```

## Demo (one command per surface)

```
target/release/agent-context print '**/*.md' --cwd bench/tree          # native CLI
.venv/bin/agent-context-py    print '**/*.md' --cwd bench/tree          # Python FFI CLI
node ffi/node/bin.js          print '**/*.md' --cwd bench/tree          # Node FFI CLI
.venv/bin/python wasm/py/cli.py print '**/*.md' --cwd bench/tree        # Python WASM CLI
node wasm/node/cli.mjs        print '**/*.md' --cwd bench/tree          # Node WASM CLI
```

SDK use is shown in `bench/bench.py` / `bench/bench.mjs` (FFI import, WASM
instantiate + preopen).

## Bench

```
python3 bench/gen_tree.py        # deterministic 30-file, ~230KB docs tree
.venv/bin/python bench/bench.py  # Python: pure-py + subprocess baselines, FFI, WASM
node bench/bench.mjs             # Node: subprocess baseline, FFI, WASM
cargo run --release -p agent-context-core --example warm_bench -- bench/tree
```

Each harness prints warm/cold medians plus a sha256 of the output text — all
surfaces must print the same hash (byte-identical output, thesis 3).
