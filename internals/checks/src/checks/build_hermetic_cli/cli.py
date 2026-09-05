"""Build the release binary and `packages/node`'s `dist/` from HEAD and stage them as the hermetic CLI artifact."""
from __future__ import annotations

import click

from checks.utils.build_hermetic_cli import stage_hermetic_cli

# The build commands, in order, as (argv, cwd relative to the checkout root).
COMMANDS = [
    (
        ["cargo", "build", "--release", "--manifest-path", "packages/rust/Cargo.toml", "--bin", "testing-conventions"],
        ".",
    ),
    (["pnpm", "install", "--no-frozen-lockfile"], "packages/node"),
    (["pnpm", "run", "build"], "packages/node"),
]

BINARY = "packages/rust/target/release/testing-conventions"
NODE_DIST = "packages/node/dist"


@click.command()
@click.argument("stage_dir", default="hermetic-cli-stage", type=click.Path())
def cli(stage_dir: str) -> None:
    stage_hermetic_cli(COMMANDS, BINARY, NODE_DIST, stage_dir)
    click.echo(f"staged the hermetic CLI artifact at {stage_dir}")
