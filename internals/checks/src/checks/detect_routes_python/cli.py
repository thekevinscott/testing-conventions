"""The detect-routes-python check — repo-only (#189; epic #302, #305, #324).

Backs the `tc-checks detect-routes-python` subcommand: the self-test workflow runs the repo-only
`detect` composite action over the `clean` fixture and hands this check the action's
`isolation_languages` output — a compact JSON array such as `["python"]` or `["python","rust"]`
— which this check asserts routes Python into the unit-lint matrix.

The value arrives as a CLI argument (never an environment side-channel: the workflow templates
`${{ steps.detect.outputs.isolation_languages }}` into the argument, single-quoted). A standalone,
colocated-tested check rather than inline `run: |` bash keeps the routing assertion off the
untested, `${{ }}`-templated `run:` path.
"""
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
