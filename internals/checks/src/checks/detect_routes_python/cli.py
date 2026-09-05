"""Assert detect's `isolation_languages` output routes Python into the unit-lint matrix."""
from __future__ import annotations

import click

from checks.detect_routes_python.routes import routes_python
from checks.utils.check_failed import CheckFailed

DEFAULT_ISOLATION_LANGUAGES = "[]"


@click.command()
@click.argument("isolation_languages", default=DEFAULT_ISOLATION_LANGUAGES, type=str)
def cli(isolation_languages: str) -> None:
    if not routes_python(isolation_languages):
        raise CheckFailed(
            "the detect action did not route Python into isolation_languages "
            f"(got: {isolation_languages})"
        )
    click.echo(f"isolation_languages={isolation_languages}")
    click.echo("Python routed into the unit-lint matrix")
