#!/usr/bin/env python3
"""Latency harness (Python side): FFI vs WASM vs pure-Python vs subprocess.

Warm: median of N calls in one process over bench/tree.
Cold: median of fresh-process runs (includes interpreter/runtime startup).
Run inside spike/.venv after `maturin develop --release` and the wasm build.
"""

import hashlib
import json
import statistics
import subprocess
import sys
import time
from pathlib import Path

SPIKE = Path(__file__).resolve().parents[1]
TREE = str(SPIKE / "bench" / "tree")
NATIVE = str(SPIKE / "target" / "release" / "agent-context")
GLOBS = ["**/*.md"]
WARM_N = 1000
COLD_N = 20

sys.path.insert(0, str(SPIKE / "bench"))
sys.path.insert(0, str(SPIKE / "wasm" / "py"))


def median_ms(fn, n):
    times = []
    for _ in range(n):
        t0 = time.perf_counter()
        fn()
        times.append((time.perf_counter() - t0) * 1000)
    return statistics.median(times)


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()[:12]


def main():
    results = {}

    # --- pure-Python baseline ---
    from baseline_pure import print_tree
    results["pure-py warm"] = median_ms(lambda: print_tree(GLOBS, TREE), WARM_N)
    pure = print_tree(GLOBS, TREE)

    # --- subprocess-over-native-binary baseline ---
    def sub():
        return subprocess.run(
            [NATIVE, "print", *GLOBS, "--cwd", TREE],
            capture_output=True, check=True, text=True,
        ).stdout
    results["subprocess warm=cold"] = median_ms(sub, COLD_N)
    sub_text = sub()

    # --- FFI (pyo3) ---
    import agent_context as ffi
    results["ffi warm"] = median_ms(lambda: ffi.print(GLOBS, TREE), WARM_N)
    ffi_r = ffi.print(GLOBS, TREE)
    cold_cmd = (
        f"import agent_context; agent_context.print({GLOBS!r}, {TREE!r})"
    )
    results["ffi cold (fresh python)"] = median_ms(
        lambda: subprocess.run([sys.executable, "-c", cold_cmd], check=True),
        COLD_N,
    )

    # --- WASM (wasmtime-py) ---
    from sdk import AgentContext
    ac = AgentContext(TREE)
    results["wasm warm"] = median_ms(lambda: ac.print(GLOBS), WARM_N)
    wasm_r = ac.print(GLOBS)
    results["wasm re-instantiate+call (in-process)"] = median_ms(
        lambda: AgentContext(TREE).print(GLOBS), COLD_N
    )
    wasm_cold = (
        f"import sys; sys.path.insert(0, {str(SPIKE / 'wasm' / 'py')!r}); "
        f"from sdk import AgentContext; AgentContext({TREE!r}).print({GLOBS!r})"
    )
    results["wasm cold (fresh python)"] = median_ms(
        lambda: subprocess.run([sys.executable, "-c", wasm_cold], check=True),
        COLD_N,
    )

    # --- interpreter-startup floor for cold context ---
    results["python noop (fresh python)"] = median_ms(
        lambda: subprocess.run([sys.executable, "-c", "pass"], check=True),
        COLD_N,
    )

    print(json.dumps({k: round(v, 3) for k, v in results.items()}, indent=2))
    print("parity sha256/12:", {
        "pure-py": sha(pure["text"]),
        "subprocess": sha(sub_text),
        "ffi": sha(ffi_r.text),
        "wasm": sha(wasm_r.text),
    })


if __name__ == "__main__":
    main()
