"""Build and stage the hermetic CLI artifact — repo-only (#356, epic #353).

Backs the `tc-checks build-hermetic-cli` subcommand: the single `run:` line of the
`./.github/actions/build-hermetic-cli` composite action (`.github/actions/build-hermetic-cli/action.yml`),
shared by the `build-cli` job in the repo-only caller workflows (testing-conventions-selftest.yml,
dogfood.yml). `COMMANDS` builds the release binary from HEAD (the same binary
`packages/node/scripts/build.ts` stages for the npm packages) and `packages/node`'s `dist/` (the TS
mutation adapter the npm launcher normally hands the binary); the layout matches detect.py's
`cli_command` / `ts_mutation_adapter_args`.

A standalone, colocated-tested command rather than inline `run: |` bash (AGENTS.md, "Logic lives
in scripts, not workflow YAML"): the orchestration (and its `run`/`root` injection seams) lives in
the shared `checks.utils.build_hermetic_cli.stage_hermetic_cli`, the `run_checks` pattern (#328) —
this module stays a thin declarative wrapper naming this check's own commands, like
`isolation_red.cli` wraps `run_checks`.
"""
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
