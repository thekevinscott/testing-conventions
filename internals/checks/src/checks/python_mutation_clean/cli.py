"""Drive the Python mutation gate over a mutation-clean fixture; the run must pass."""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "unit", "mutation", "--language", "python", ".github/selftest/mutation/python-clean"],
        False,
        "clean Python library passes unit mutation",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
