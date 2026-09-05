"""Drive the integration lint over the Rust and TypeScript arms; the violating suites fail and the
clean one passes."""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "integration", "lint", "--language", "rust", ".github/selftest/integration-rust/red"],
        True,
        "red Rust crate trips integration lint",
    ),
    (
        [*HERMETIC_CLI, "integration", "lint", "--language", "typescript", ".github/selftest/integration-typescript/red"],
        True,
        "red TypeScript suite trips integration lint",
    ),
    (
        [*HERMETIC_CLI, "integration", "lint", "--language", "typescript", ".github/selftest/integration-typescript/clean"],
        False,
        "clean TypeScript suite passes integration lint",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
