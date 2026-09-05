"""A check failure: a `click.ClickException` whose `show` prints a GitHub Actions `::error::`
annotation."""
from __future__ import annotations

import click


class CheckFailed(click.ClickException):
    """Exit non-zero with a GitHub Actions `::error::` annotation carrying the message."""

    def show(self, file: object = None) -> None:
        click.echo(f"::error::{self.message}")
