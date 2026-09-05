"""Assert detect's outputs for the monorepo Python fixture: its own directory as package root, uv as
the Python environment, and its own `testing-conventions.toml` as config."""
from __future__ import annotations

import click

from checks.detect_package_root_py.evaluate import evaluate
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("package_root")
@click.argument("python_env")
@click.argument("config")
def cli(package_root: str, python_env: str, config: str) -> None:
    error = evaluate(package_root, python_env, config)
    if error:
        raise CheckFailed(error)
    click.echo(f"package_root={package_root} python_env={python_env} config={config}")
