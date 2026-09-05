"""Run each `(argv, expect_fail, label)` through the injected `run`, echo every check that holds, and
raise `CheckFailed` on the first that does not."""
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
