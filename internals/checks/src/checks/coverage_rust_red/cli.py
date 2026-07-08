"""The coverage-rust-red check — repo-only (#302 #309, #321, #328).

Backs the `tc-checks coverage-rust-red` subcommand: the `coverage-rust-red` job in
`.github/workflows/testing-conventions-selftest.yml` drives the hermetic-CLI (built-from-HEAD) `unit coverage --language rust` command over a sub-floor crate and asserts the non-zero exit that fails a consumer's build (#302 #309).

A standalone, colocated-tested check rather than an inline `run: |` bash block: inline workflow
bash is untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:`
text is templated before the shell sees it).

The workflow step runs the hardcoded `CHECKS` through the shared `run_checks` orchestrator; a
trailing command may be passed instead (a single red-path check — the seam the e2e suite drives
with a benign `true`/`false`).
"""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "unit", "coverage", "--language", "rust", "--config", ".github/selftest/coverage-rust/below/testing-conventions.toml", ".github/selftest/coverage-rust/below"],
        True,
        "below-floor crate trips the rust coverage gate",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
