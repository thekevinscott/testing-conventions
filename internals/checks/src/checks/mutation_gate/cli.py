"""The mutation-gate check — repo-only (#302 #309, #321, #328).

Backs the `tc-checks mutation-gate` subcommand: the `mutation-gate` job in
`.github/workflows/testing-conventions-selftest.yml` drives the published `unit mutation --language rust` command over a clean crate that must pass and a survivor crate that must fail, and asserts each exit code (#302 #309).

A standalone, colocated-tested check rather than an inline `run: |` bash block: inline workflow
bash is untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:`
text is templated before the shell sees it).

The workflow step runs the hardcoded `CHECKS` through the shared `run_checks` orchestrator; a
trailing command may be passed instead (a single red-path check — the seam the e2e suite drives
with a benign `true`/`false`).
"""
from __future__ import annotations

import click

from checks.utils.run_checks import run_checks

CHECKS = [
    (
        ["npx", "-y", "testing-conventions", "unit", "mutation", "--language", "rust", ".github/selftest/mutation/clean"],
        False,
        "clean crate passes unit mutation",
    ),
    (
        ["npx", "-y", "testing-conventions", "unit", "mutation", "--language", "rust", ".github/selftest/mutation/survivor"],
        True,
        "survivor crate trips the mutation gate",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
