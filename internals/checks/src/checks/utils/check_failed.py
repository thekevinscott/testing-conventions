"""A check failure — repo-only (#321).

Raised by a check when the wiring it asserts is missing. It is a `click.ClickException`, so click
exits non-zero and calls `show`; `show` prints a GitHub Actions `::error::` annotation.
"""
from __future__ import annotations

import click


class CheckFailed(click.ClickException):
    """Exit non-zero with a GitHub Actions `::error::` annotation carrying the message."""

    def show(self, file: object = None) -> None:
        click.echo(f"::error::{self.message}")
