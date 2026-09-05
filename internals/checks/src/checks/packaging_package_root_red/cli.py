"""Drive the packaging gate over a per-package `dist/` tarball shipping a test file; the run must fail."""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "packaging", ".github/selftest/packaging-package-root/red/dist/packaging-package-root-fixture-0.0.0.tgz", "--language", "typescript"],
        True,
        "red per-package dist tarball trips the packaging check",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
