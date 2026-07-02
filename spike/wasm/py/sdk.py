"""WASM-track Python SDK over wasmtime-py's dynamic component API.

wasmtime-py 46 has no `wasmtime.bindgen` (the old generated-bindings module
is gone); components are driven dynamically: `Linker.add_wasip2()` supplies
the WASI host, `WasiConfig.preopen_dir` grants filesystem capability, and
exports are looked up per interface then per function.
"""

from pathlib import Path

from wasmtime import Engine, Store, WasiConfig
from wasmtime.component import Component, Linker

WASM = str(
    Path(__file__).resolve().parents[2]
    / "target/wasm32-wasip2/release/agent_context_wasm.wasm"
)
WORLD = "spike:agent-context/tool"
GUEST_DIR = "/work"


class AgentContext:
    """Instantiate once, call many (the SDK reality)."""

    def __init__(self, host_dir: str, wasm: str = WASM, guest_dir: str = GUEST_DIR):
        self.guest_dir = guest_dir
        self.engine = Engine()
        self.component = Component.from_file(self.engine, wasm)
        linker = Linker(self.engine)
        linker.add_wasip2()
        self.store = Store(self.engine)
        wasi = WasiConfig()
        wasi.preopen_dir(host_dir, guest_dir)
        wasi.inherit_stdout()
        wasi.inherit_stderr()
        self.store.set_wasi(wasi)
        self.instance = linker.instantiate(self.store, self.component)
        tool = self.instance.get_export_index(self.store, WORLD)
        self._fns = {
            name: self.instance.get_func(
                self.store,
                self.instance.get_export_index(self.store, name, tool),
            )
            for name in ("print", "write-block", "crash", "run")
        }

    def _call(self, name, *args):
        # The dynamic API does NOT discriminate result<T, string>: it returns
        # the ok payload directly on success and the err string directly on
        # failure. `run` is the only export whose ok arm isn't distinguishable
        # by type from a string — it returns s32, so type-sniffing holds here,
        # but this is a real wart (an ok arm of `string` would be ambiguous).
        result = self._fns[name](self.store, *args)
        if name != "run" and isinstance(result, str):
            raise ValueError(result)
        return result

    def print(self, globs, cwd=None):
        return self._call("print", globs, cwd or self.guest_dir)

    def write_block(self, path, id, content):
        return self._call("write-block", path, id, content)

    def crash(self, mode):
        return self._call("crash", mode)

    def run(self, argv):
        return self._call("run", argv)
