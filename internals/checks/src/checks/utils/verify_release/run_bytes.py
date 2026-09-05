"""Run a command through the injected `run`, returning raw stdout bytes."""
from __future__ import annotations

from checks.utils.verify_release.ensure_ok import ensure_ok


def run_bytes(run, argv: list[str], **extra) -> bytes:
    result = run(argv, capture_output=True, **extra)
    ensure_ok(result, argv)
    return result.stdout
