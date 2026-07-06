"""The coverage-rust-red check — repo-only (#302 #309, #321, #328).

Backs the `tc-checks coverage-rust-red` subcommand: the `coverage-rust-red` job in
`.github/workflows/testing-conventions-selftest.yml` drives the published `unit coverage --language rust` command over a sub-floor crate and asserts the non-zero exit that fails a consumer's build (#302 #309).

A standalone, colocated-tested check rather than an inline `run: |` bash block: inline workflow
bash is untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:`
text is templated before the shell sees it).

The subprocess boundary lives behind `run_command`; the pure exit-code decision is `failure_reason`.
The workflow step runs the hardcoded `CHECKS`; a trailing command may be passed instead (a single
red-path check — the seam the e2e suite drives with a benign `true`/`false`).
"""
from __future__ import annotations

import subprocess
from typing import Optional

import click

from checks.utils.check_failed import CheckFailed

# The published-CLI invocations this job asserts on. Each entry is `(command, expect_fail, label)`:
# `expect_fail` is True for a red-path check (the command must exit non-zero) and False for a
# clean-path check (it must exit zero).
CHECKS = [
    (
        ["npx", "-y", "testing-conventions", "unit", "coverage", "--language", "rust", "--config", ".github/selftest/coverage-rust/below/testing-conventions.toml", ".github/selftest/coverage-rust/below"],
        True,
        "below-floor crate trips the rust coverage gate",
    ),
]


def run_command(command):
    """Run `command`, streaming its output, and return the CompletedProcess. The one external
    boundary; the e2e suite drives it for real against a benign command."""
    return subprocess.run(command)


def failure_reason(expect_fail: bool, returncode: int) -> Optional[str]:
    """The pure exit-code decision: `None` when `returncode` met the expectation, else the reason
    it did not. Truthiness — not `== 0` — so no comparison operator can mutate into an equivalent
    (`<= 0` never differs for a real exit code; `is 0` never differs for a small cached int)."""
    if expect_fail:
        if not returncode:
            return "the command exited 0, but a non-zero (failing) exit was required"
        return None
    if returncode:
        return f"the command exited {returncode}, but a zero (passing) exit was required"
    return None


def checks_for(command) -> list:
    """The checks to run: a single red-path check for a given trailing `command`, else the hardcoded
    `CHECKS`. The e2e seam drives the former with a benign command; the truthiness of `command`
    carries no comparison operator to mutate."""
    if command:
        return [(list(command), True, "cli")]
    return CHECKS


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    failures = []
    for cmd, expect_fail, label in checks_for(command):
        reason = failure_reason(expect_fail, run_command(cmd).returncode)
        if reason is None:
            click.echo(f"[{label}] ok")
        else:
            failures.append(f"[{label}] {reason}")
    if failures:
        raise CheckFailed("; ".join(failures))
