"""Run a command through the injected `run`, returning decoded stdout."""
from __future__ import annotations

from checks.utils.verify_release.ensure_ok import ensure_ok


def run_text(run, argv: list[str]) -> str:
    result = run(argv, capture_output=True, text=True)
    ensure_ok(result, argv)
    return result.stdout
