"""Drive the colocated-test gate over a Rust crate with no inline `#[cfg(test)]`; the run must fail."""
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
