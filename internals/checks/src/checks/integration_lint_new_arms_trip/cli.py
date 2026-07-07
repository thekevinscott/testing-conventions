"""The integration-lint-new-arms-trip check — repo-only (#302 #309, #321, #328).

Backs the `tc-checks integration-lint-new-arms-trip` subcommand: the `integration-lint-new-arms-trip` job in
`.github/workflows/testing-conventions-selftest.yml` drives the published `integration lint` command over the Rust and TypeScript arms — a violating crate and suite that must fail, and a clean TypeScript suite that must pass — and asserts each exit code (#126 #302 #309).

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
        ["npx", "-y", "testing-conventions", "integration", "lint", "--language", "rust", ".github/selftest/integration-rust/red"],
        True,
        "red Rust crate trips integration lint",
    ),
    (
        ["npx", "-y", "testing-conventions", "integration", "lint", "--language", "typescript", ".github/selftest/integration-typescript/red"],
        True,
        "red TypeScript suite trips integration lint",
    ),
    (
        ["npx", "-y", "testing-conventions", "integration", "lint", "--language", "typescript", ".github/selftest/integration-typescript/clean"],
        False,
        "clean TypeScript suite passes integration lint",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
