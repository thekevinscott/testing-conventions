"""Drive the Rust mutation gate over a clean crate and a survivor crate; only the survivor fails."""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "unit", "mutation", "--language", "rust", ".github/selftest/mutation/clean"],
        False,
        "clean crate passes unit mutation",
    ),
    (
        [*HERMETIC_CLI, "unit", "mutation", "--language", "rust", ".github/selftest/mutation/survivor"],
        True,
        "survivor crate trips the mutation gate",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
