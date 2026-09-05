"""Drive the packaging gate over a wheel shipping a `*_test.py`; the run must fail."""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "packaging", ".github/selftest/packaging/red/widget-0.1.0-py3-none-any.whl", "--language", "python"],
        True,
        "red wheel trips the packaging check",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
