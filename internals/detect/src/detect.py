#!/usr/bin/env python3
"""Entry point for the reusable workflow's `detect` job; sibling modules hold the derivations.

The composite action passes one argument per input, in `ARGUMENTS` order, and the detected sets
are appended to the file `GITHUB_OUTPUT` names. Every argument is required; an empty one is a value.
"""
import os
import sys

from compute_outputs import compute_outputs
from render_github_output import render_github_output

ARGUMENTS = ("languages", "scan_path", "config", "caller_repository", "version")


def main(argv) -> int:
    arguments = dict(zip(ARGUMENTS, argv))
    missing = [name for name in ARGUMENTS if name not in arguments]
    if missing:
        order, absent = ", ".join(ARGUMENTS), ", ".join(missing)
        print(
            f"::error::detect.py takes an argument per input, in order: {order}."
            f" Missing: {absent}",
            file=sys.stderr,
        )
        return 1

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
