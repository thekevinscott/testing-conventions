"""Validated-promotion steps for the verify-and-promote workflow — repo-only (#357, #321).

Backs the `tc-checks verify-release` group: the three steps `move-major-tag.yml` runs before
advancing `@v0` — resolve the just-published version, check the remote `detect`-action fetch
layout at the release commit, and dispatch + poll the version-pinned self-test + dogfood suites.
Each subcommand is a thin wrapper over `checks.utils.verify_release`, where every decision and
parse is unit-tested against an injected git/gh boundary; the wrappers themselves are the
exempted glue (like `build_hermetic_cli.cli`), so a real run drives real git/gh.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap.
"""
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
