"""Build and stage the hermetic CLI artifact — repo-only (#356, epic #353).

Backs the `tc-checks build-hermetic-cli` subcommand: the single `run:` line of the `build-cli`
job in the repo-only caller workflows (testing-conventions-selftest.yml, dogfood.yml). Builds
the release binary from HEAD (the same binary `packages/node/scripts/build.ts` stages for the
npm packages), builds packages/node's `dist/` (the TS mutation adapter the npm launcher normally
hands the binary), and stages both in the layout detect.py's `cli_command` /
`ts_mutation_adapter_args` name: the binary at the stage root, `dist/` beside it. The exec bit is
set at staging because artifact upload/download preserves paths, not modes — the workflow's
download step re-chmods on the other side.

A standalone, colocated-tested command rather than inline `run: |` bash (AGENTS.md, "Logic lives
in scripts, not workflow YAML"): the subprocess boundary is injected as `run`, the `run_checks`
pattern (#328), so the colocated test drives a fake with no real subprocess.
"""
from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import click

from checks.utils.check_failed import CheckFailed

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
def cli(stage_dir: str, root: str = ".", run=subprocess.run) -> None:
    root_path = Path(root)
    for argv, cwd in COMMANDS:
        result = run(argv, cwd=str(root_path / cwd))
        if result.returncode != 0:
            raise CheckFailed(f"`{' '.join(argv)}` exited {result.returncode}")
    stage = Path(stage_dir)
    stage.mkdir(parents=True, exist_ok=True)
    staged_binary = stage / "testing-conventions"
    shutil.copyfile(root_path / BINARY, staged_binary)
    staged_binary.chmod(0o755)
    shutil.copytree(root_path / NODE_DIST, stage / "dist", dirs_exist_ok=True)
    click.echo(f"staged the hermetic CLI artifact at {stage_dir}")
