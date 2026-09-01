"""Backs `tc-checks verify-release`: the three steps `move-major-tag.yml` runs before advancing
`@v0` — resolve the published version, check the remote `detect` fetch layout, and dispatch and
poll the version-pinned self-test and dogfood suites."""
from __future__ import annotations

import click

from checks.utils import verify_release as vr
from checks.utils.check_failed import CheckFailed


@click.group()
def cli() -> None:
    """Validated-promotion steps run before `@v0` advances (#357)."""


@cli.command("resolve-version")
@click.argument("sha")
def resolve_version(sha: str) -> None:
    click.echo(vr.resolve_version(sha))


@cli.command("check-layout")
@click.argument("sha")
def check_layout(sha: str) -> None:
    absent = vr.check_layout(sha)
    if absent:
        raise CheckFailed(vr.layout_error(sha, absent))
    click.echo(vr.layout_ok(sha))


@cli.command("dispatch-and-wait")
@click.argument("sha")
@click.argument("version")
@click.argument("workflows", nargs=-1)
def dispatch_and_wait(sha: str, version: str, workflows: tuple[str, ...]) -> None:
    conclusions = vr.verify_suites(sha, version, list(workflows))
    failed = vr.failed_suites(conclusions)
    if failed:
        raise CheckFailed(vr.verification_error(sha, failed))
    click.echo(vr.verification_ok(sha, conclusions))
