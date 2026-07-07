"""The python-mutation-clean check — repo-only (#302 #309, #321, #328).

Backs the `tc-checks python-mutation-clean` subcommand: the `python-mutation-clean` job in
`.github/workflows/testing-conventions-selftest.yml` drives the published `unit mutation --language python` command over a mutation-clean fixture and asserts the zero exit that proves the wheel-shipped adapter resolves (#258).

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
        ["npx", "-y", "testing-conventions", "unit", "mutation", "--language", "python", ".github/selftest/mutation/python-clean"],
        False,
        "clean Python library passes unit mutation",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
