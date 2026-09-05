"""The dispatch-and-wait step of `tc-checks verify-release`."""
from __future__ import annotations

import click

from checks.utils.check_failed import CheckFailed
from checks.utils.verify_release.failed_suites import failed_suites
from checks.utils.verify_release.verification_error import verification_error, verification_ok
from checks.utils.verify_release.verify_suites import verify_suites


@click.command("dispatch-and-wait")
@click.argument("sha")
@click.argument("version")
@click.argument("workflows", nargs=-1)
def dispatch_and_wait(sha: str, version: str, workflows: tuple[str, ...]) -> None:
    conclusions = verify_suites(sha, version, list(workflows))
    failed = failed_suites(conclusions)
    if failed:
        raise CheckFailed(verification_error(sha, failed))
    click.echo(verification_ok(sha, conclusions))
