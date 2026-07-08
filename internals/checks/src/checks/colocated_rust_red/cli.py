"""The colocated-rust-red check — repo-only (#274, #379).

Backs the `tc-checks colocated-rust-red` subcommand: the `colocated-rust-red` job in
`.github/workflows/testing-conventions-selftest.yml` drives the hermetic-CLI (built-from-HEAD)
`unit colocated-test --language rust` command over a crate whose bodied `fn` has no inline
`#[cfg(test)]` module and asserts the non-zero exit that fails a consumer's build (#274). Moved
off the inline `run: |` bash the job carried into a tested check (#379), which also routes it
through the hermetic binary rather than the published npx.

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
        [*HERMETIC_CLI, "unit", "colocated-test", "--language", "rust", ".github/selftest/colocated-rust/red"],
        True,
        "orphan crate trips the rust colocated-test arm",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
