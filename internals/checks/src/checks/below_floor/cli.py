"""Drive the coverage gate over a sub-floor Python suite; the run must fail."""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "unit", "coverage", "--language", "python", "--config", ".github/selftest/below-floor/testing-conventions.toml", ".github/selftest/below-floor"],
        True,
        "below-floor suite trips the coverage gate",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
