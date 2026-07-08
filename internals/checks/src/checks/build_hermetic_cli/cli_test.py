"""Colocated unit tests for the build-hermetic-cli check (isolation — no CliRunner, no real
subprocess).

Imports only the unit under test: this check's own `COMMANDS`/`BINARY`/`NODE_DIST` data and `cli`
command's declared argument. The shared staging orchestration and its `run`/`root` injection seams
are covered once beside `stage_hermetic_cli` (checks/utils/build_hermetic_cli_test.py) — the same
split `isolation_red.cli_test` uses for `run_checks`: `cli` itself is a thin, unconditional
passthrough with no branch of its own to exercise here.
"""
from checks.build_hermetic_cli.cli import BINARY, COMMANDS, NODE_DIST, cli


def test_commands_are_the_expected_builds():
    assert COMMANDS == [
        (
            ["cargo", "build", "--release", "--manifest-path", "packages/rust/Cargo.toml", "--bin", "testing-conventions"],
            ".",
        ),
        (["pnpm", "install", "--no-frozen-lockfile"], "packages/node"),
        (["pnpm", "run", "build"], "packages/node"),
    ]


def test_binary_and_node_dist_are_the_release_build_outputs():
    assert BINARY == "packages/rust/target/release/testing-conventions"
    assert NODE_DIST == "packages/node/dist"


def test_declares_the_stage_dir_argument_with_its_default():
    (argument,) = cli.params
    assert argument.name == "stage_dir"
    assert argument.default == "hermetic-cli-stage"
