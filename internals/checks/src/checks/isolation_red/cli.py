"""Drive the isolation lint over an un-isolated Rust crate; the run must fail."""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "unit", "lint", "--language", "rust", ".github/selftest/isolation/rust-red"],
        True,
        "un-isolated crate trips unit lint",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
