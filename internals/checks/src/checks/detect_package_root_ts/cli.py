"""Assert detect's outputs for the monorepo TS fixture: its own directory as package root, npm as the
package manager, no Rust provisioning, and its own `testing-conventions.toml` as config."""
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
    error = evaluate(package_root, ts_package_manager, provision_rust, config)
    if error:
        raise CheckFailed(error)
    click.echo(
        f"package_root={package_root} ts_package_manager={ts_package_manager} "
        f"provision_rust={provision_rust} config={config}"
    )
