"""Colocated unit tests for the build-hermetic-cli check (isolation — no CliRunner, no real
subprocess).

Imports only the unit under test: this check's own `COMMANDS`/`BINARY`/`NODE_DIST` data, the `cli`
command's declared argument, and the wiring between them, driven through `.callback()` with
`stage_hermetic_cli` patched by string target. The staging orchestration behind that seam is
covered once beside it, in `checks/utils/build_hermetic_cli_test.py`.
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


def test_stages_this_checks_own_commands_and_outputs_into_the_given_dir(monkeypatch, capsys):
    staged = []
    monkeypatch.setattr("checks.build_hermetic_cli.cli.stage_hermetic_cli", lambda *args: staged.append(args))
    cli.callback(stage_dir="somewhere")
    assert staged == [(COMMANDS, BINARY, NODE_DIST, "somewhere")]
    assert capsys.readouterr().out == "staged the hermetic CLI artifact at somewhere\n"
