#!/usr/bin/env python3
"""CLI shim over the WASM component (preopens host "/" to mirror a real CLI)."""

import sys

from sdk import AgentContext

if __name__ == "__main__":
    ac = AgentContext("/", guest_dir="/")
    sys.exit(ac.run(sys.argv[1:]))
