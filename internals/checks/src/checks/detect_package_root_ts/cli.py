"""The detect-package-root-ts check — repo-only (#277 #302 #321 #327).

Backs the `tc-checks detect-package-root-ts` subcommand: the self-test
(`.github/workflows/testing-conventions-selftest.yml`) runs `./.github/actions/detect` against
`.github/selftest/monorepo/packages/ts/src` with only `languages`/`path`, then asserts that
package-root discovery (#277) resolves the fixture's own directory, npm as the TS package manager,
no Rust provisioning, and the fixture's own `testing-conventions.toml` as config. The detect
outputs arrive as CLI arguments (never an env side-channel).

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` is silently evaluated).
"""
from __future__ import annotations

import click

from checks.detect_package_root_ts.evaluate import evaluate
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("package_root")
@click.argument("ts_package_manager")
@click.argument("provision_rust")
@click.argument("config")
def cli(package_root: str, ts_package_manager: str, provision_rust: str, config: str) -> None:
    """Assert detect's outputs for the monorepo TS fixture (#277)."""
    error = evaluate(package_root, ts_package_manager, provision_rust, config)
    if error:
        raise CheckFailed(error)
    click.echo(
        f"package_root={package_root} ts_package_manager={ts_package_manager} "
        f"provision_rust={provision_rust} config={config}"
    )
