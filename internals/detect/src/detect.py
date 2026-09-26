#!/usr/bin/env python3
"""Entry point for the reusable workflow's `detect` job; sibling modules hold the derivations.

The composite action passes the five inputs positionally, in `ARGUMENTS` order, and the detected
sets are appended to the file `GITHUB_OUTPUT` names.
"""
import os
import sys

from compute_outputs import compute_outputs
from derive_config import CONFIG_DEFAULT
from render_github_output import render_github_output

ARGUMENTS = {
    "languages": "",
    "scan_path": ".",
    "config": CONFIG_DEFAULT,
    "caller_repository": "",
    "version": "",
}


def main(argv) -> int:
    supplied = dict(zip(ARGUMENTS, argv))
    arguments = {name: supplied.get(name, default) for name, default in ARGUMENTS.items()}
    outputs = compute_outputs(
        arguments["languages"],
        arguments["scan_path"],
        config_input=arguments["config"],
        caller_repository=arguments["caller_repository"],
        version=arguments["version"],
    )

    github_output = os.environ.get("GITHUB_OUTPUT")
    if github_output:
        with open(github_output, "a", encoding="utf-8") as handle:
            handle.write(render_github_output(outputs))
    summary = ", ".join(f"{name} {value}" for name, value in outputs.items())
    languages, scan_path = arguments["languages"], arguments["scan_path"]
    print(f"languages='{languages}' under '{scan_path}' -> {summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
