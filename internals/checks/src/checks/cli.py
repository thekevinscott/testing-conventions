"""The single console entry point: the click group that composes the checks (#321).

Repo-only. `tc-checks` is one command; each check is a `@click.command()` in its own subfolder,
registered here as a subcommand. Adding a check is a folder plus one `add_command` line.
"""
from __future__ import annotations

import click

from checks.build_command_wired.cli import cli as build_command_wired
from checks.e2e_verify_scope_wired.cli import cli as e2e_verify_scope_wired
from checks.gates_wired.cli import cli as gates_wired
from checks.rust_toolchain_wired.cli import cli as rust_toolchain_wired


@click.group()
def main() -> None:
    """Repo-only self-test wiring/assertion checks (#321)."""


main.add_command(build_command_wired, name="build-command-wired")
main.add_command(e2e_verify_scope_wired, name="e2e-verify-scope-wired")
main.add_command(gates_wired, name="gates-wired")
main.add_command(rust_toolchain_wired, name="rust-toolchain-wired")
