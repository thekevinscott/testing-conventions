"""Colocated unit tests for the build-hermetic-cli command (isolation — no CliRunner, no real
subprocess: the boundary is the injected `run`, the run_checks pattern, #328)."""
from checks.build_hermetic_cli.cli import BINARY, COMMANDS, NODE_DIST, cli


class _Result:
    def __init__(self, returncode):
        self.returncode = returncode


def _root(tmp_path):
    binary = tmp_path / BINARY
    binary.parent.mkdir(parents=True)
    binary.write_bytes(b"binary")
    adapter = tmp_path / NODE_DIST / "mutation" / "main.js"
    adapter.parent.mkdir(parents=True)
    adapter.write_text("adapter")
    return tmp_path


def test_commands_are_the_expected_builds():
    assert COMMANDS == [
        (
            ["cargo", "build", "--release", "--manifest-path", "packages/rust/Cargo.toml", "--bin", "testing-conventions"],
            ".",
        ),
        (["pnpm", "install", "--no-frozen-lockfile"], "packages/node"),
        (["pnpm", "run", "build"], "packages/node"),
    ]


def test_runs_the_build_commands_in_order_then_stages(tmp_path, capsys):
    calls = []

    def run(argv, cwd):
        calls.append((argv, cwd))
        return _Result(0)

    root = _root(tmp_path)
    stage = tmp_path / "stage"
    cli.callback(stage_dir=str(stage), root=str(root), run=run)
    assert [argv for argv, _ in calls] == [argv for argv, _ in COMMANDS]
    assert calls[0][1] == str(root / ".")
    assert calls[1][1] == str(root / "packages/node")
    staged = stage / "testing-conventions"
    assert staged.read_bytes() == b"binary"
    # Exec bit set at staging: artifact upload/download preserves paths, not modes. Exact
    # permission bits, not just truthy — a partial-exec mode (e.g. owner+group only) would
    # still be truthy against a loose `& 0o111` check.
    assert staged.stat().st_mode & 0o777 == 0o755
    assert (stage / "dist" / "mutation" / "main.js").read_text() == "adapter"
    assert "staged the hermetic CLI artifact" in capsys.readouterr().out


def test_stages_into_a_stage_dir_with_missing_parent_directories(tmp_path):
    # mkdir(parents=True): the stage dir's own parents may not exist yet.
    root = _root(tmp_path)
    stage = tmp_path / "nested" / "missing" / "stage"

    def run(argv, cwd):
        return _Result(0)

    cli.callback(stage_dir=str(stage), root=str(root), run=run)
    assert (stage / "testing-conventions").read_bytes() == b"binary"


def test_stages_into_an_already_existing_stage_dir(tmp_path):
    # mkdir(exist_ok=True): a rerun (or a stage dir some earlier step already created) must
    # not raise FileExistsError.
    root = _root(tmp_path)
    stage = tmp_path / "stage"
    stage.mkdir(parents=True)

    def run(argv, cwd):
        return _Result(0)

    cli.callback(stage_dir=str(stage), root=str(root), run=run)
    assert (stage / "testing-conventions").read_bytes() == b"binary"


def test_stages_dist_over_an_already_existing_dist_dir(tmp_path):
    # copytree(dirs_exist_ok=True): a rerun (or a stage/dist an earlier step already created)
    # must merge rather than raise FileExistsError.
    root = _root(tmp_path)
    stage = tmp_path / "stage"
    (stage / "dist").mkdir(parents=True)

    def run(argv, cwd):
        return _Result(0)

    cli.callback(stage_dir=str(stage), root=str(root), run=run)
    assert (stage / "dist" / "mutation" / "main.js").read_text() == "adapter"


def test_raises_when_a_build_command_fails_and_stages_nothing(tmp_path):
    def run(argv, cwd):
        return _Result(2)

    try:
        cli.callback(stage_dir=str(tmp_path / "stage"), root=str(tmp_path), run=run)
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "cargo build" in error.message
        assert "exited 2" in error.message
    else:
        raise AssertionError("a failing build command must raise")
    assert not (tmp_path / "stage").exists()


def test_raises_when_a_build_command_is_killed_by_a_signal(tmp_path):
    # A negative returncode (POSIX signal death, e.g. OOM-killed cargo) must be treated as a
    # failure too — `!= 0`, not `> 0`, which a negative returncode would silently pass.
    def run(argv, cwd):
        return _Result(-9)

    try:
        cli.callback(stage_dir=str(tmp_path / "stage"), root=str(tmp_path), run=run)
    except Exception as error:  # noqa: BLE001
        assert "exited -9" in error.message
    else:
        raise AssertionError("a signal-killed build command must raise")


def test_declares_the_stage_dir_argument_with_its_default():
    (argument,) = cli.params
    assert argument.name == "stage_dir"
    assert argument.default == "hermetic-cli-stage"
