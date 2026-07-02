"""Naive pure-Python `print` — the latency baseline for thesis 2."""

import glob as globlib
import os


def print_tree(globs, cwd="."):
    rels = []
    for pattern in globs:
        for path in globlib.glob(os.path.join(cwd, pattern), recursive=True):
            if os.path.isfile(path):
                rels.append(os.path.relpath(path, cwd).replace("\\", "/"))
    rels = sorted(set(rels))
    parts = []
    for rel in rels:
        with open(os.path.join(cwd, rel), encoding="utf-8") as f:
            content = f.read()
        parts.append(f"===== BEGIN {rel} =====\n")
        parts.append(content)
        if not content.endswith("\n"):
            parts.append("\n")
        parts.append(f"===== END {rel} =====\n")
    return {"files": rels, "text": "".join(parts)}
