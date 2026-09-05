"""Drive the coverage gate over a sub-floor Rust crate; the run must fail."""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "unit", "coverage", "--language", "rust", "--config", ".github/selftest/coverage-rust/below/testing-conventions.toml", ".github/selftest/coverage-rust/below"],
        True,
        "below-floor crate trips the rust coverage gate",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
