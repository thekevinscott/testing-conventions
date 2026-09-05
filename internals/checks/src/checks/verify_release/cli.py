"""Backs `tc-checks verify-release`: the three steps `move-major-tag.yml` runs before advancing
`@v0` — resolve the published version, check the remote `detect` fetch layout, and dispatch and
poll the version-pinned self-test and dogfood suites."""
from __future__ import annotations

import click

from checks.utils.check_failed import CheckFailed
from checks.utils.verify_release.check_layout import check_layout as find_missing_layout_paths
from checks.utils.verify_release.layout_error import layout_error, layout_ok
from checks.utils.verify_release.resolve_version import resolve_version as resolve_published_version
from checks.verify_release.dispatch_and_wait import dispatch_and_wait


@click.group()
def cli() -> None:
    """Validated-promotion steps run before `@v0` advances."""


@cli.command("resolve-version")
@click.argument("sha")
def resolve_version(sha: str) -> None:
    click.echo(resolve_published_version(sha))


@cli.command("check-layout")
@click.argument("sha")
def check_layout(sha: str) -> None:
    absent = find_missing_layout_paths(sha)
    if absent:
        raise CheckFailed(layout_error(sha, absent))
    click.echo(layout_ok(sha))


cli.add_command(dispatch_and_wait)
