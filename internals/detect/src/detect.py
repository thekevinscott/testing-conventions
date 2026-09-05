#!/usr/bin/env python3
"""Entry point for the reusable workflow's `detect` job; sibling modules hold the derivations.

Inputs come from the environment set by the workflow — LANGUAGES, SCAN_PATH, CONFIG,
CALLER_REPOSITORY, VERSION — and the detected sets are appended to GITHUB_OUTPUT.
"""
import os

from compute_outputs import compute_outputs
from derive_config import CONFIG_DEFAULT
from render_github_output import render_github_output


def main() -> int:
    languages = os.environ.get("LANGUAGES", "")
    scan_path = os.environ.get("SCAN_PATH", ".")
    config_input = os.environ.get("CONFIG", CONFIG_DEFAULT)
    caller_repository = os.environ.get("CALLER_REPOSITORY", "")
    version = os.environ.get("VERSION", "")
    outputs = compute_outputs(
        languages,
        scan_path,
        config_input=config_input,
        caller_repository=caller_repository,
        version=version,
    )

    github_output = os.environ.get("GITHUB_OUTPUT")
    if github_output:
        with open(github_output, "a", encoding="utf-8") as handle:
            handle.write(render_github_output(outputs))
    summary = ", ".join(f"{name} {value}" for name, value in outputs.items())
    print(f"languages='{languages}' under '{scan_path}' -> {summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
