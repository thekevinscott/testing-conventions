"""The shared orchestrator for the failure-path checks (#328).

`run_checks(checks, command, run=subprocess.run)` runs each `(argv, expect_fail, label)` in
`checks` — or, when a trailing `command` is passed, that single red-path check (the e2e seam that
stands in for the real npx invocation) — through `run`, applies `failure_reason`, echoes
`[label] ok` for every check that holds, and raises `CheckFailed` on the first violation.

The subprocess boundary is injected as `run` (defaulting to `subprocess.run`), so the colocated
test drives a fake with no real subprocess, while the check commands run npx for real in CI.
"""
from __future__ import annotations

import subprocess

import click

from checks.utils.check_failed import CheckFailed
from checks.utils.failure_reason import failure_reason


def run_checks(checks, command=(), run=subprocess.run) -> None:
    if command:
        checks = [(list(command), True, "cli")]
    for argv, expect_fail, label in checks:
        reason = failure_reason(expect_fail, run(argv).returncode)
        if reason is None:
            click.echo(f"[{label}] ok")
        else:
            raise CheckFailed(f"[{label}] {reason}")
