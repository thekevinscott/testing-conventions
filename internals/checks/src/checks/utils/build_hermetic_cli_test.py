"""Colocated unit test for `stage_hermetic_cli` — the shared build-and-stage orchestrator
(isolation — no CliRunner, no real subprocess: the boundary is the injected `run`, the
`run_checks` pattern, #328).
"""
from checks.utils.build_hermetic_cli import stage_hermetic_cli


class _Result:
    def __init__(self, returncode):
        self.returncode = returncode


def _root(tmp_path, binary, node_dist):
    b = tmp_path / binary
    b.parent.mkdir(parents=True, exist_ok=True)
    b.write_bytes(b"binary")
    adapter = tmp_path / node_dist / "mutation" / "main.js"
    adapter.parent.mkdir(parents=True, exist_ok=True)
    adapter.write_text("adapter")
    return tmp_path


def test_runs_the_commands_in_order_then_stages(tmp_path, capsys):
    binary, node_dist = "packages/rust/target/release/testing-conventions", "packages/node/dist"
    commands = [(["a"], "."), (["b"], "packages/node")]
    calls = []

    def run(argv, cwd):
        calls.append((argv, cwd))
        return _Result(0)

    root = _root(tmp_path, binary, node_dist)
    stage = tmp_path / "stage"
    stage_hermetic_cli(commands, binary, node_dist, str(stage), root=str(root), run=run)
    assert [argv for argv, _ in calls] == [argv for argv, _ in commands]
    assert calls[0][1] == str(root / ".")
    assert calls[1][1] == str(root / "packages/node")
    staged = stage / "testing-conventions"
    assert staged.read_bytes() == b"binary"
    # Exec bit set at staging: artifact upload/download preserves paths, not modes. Exact
    # permission bits, not just truthy — a partial-exec mode (e.g. owner+group only) would
    # still be truthy against a loose `& 0o111` check.
    assert staged.stat().st_mode & 0o777 == 0o755
    assert (stage / "dist" / "mutation" / "main.js").read_text() == "adapter"


def test_stages_into_a_stage_dir_with_missing_parent_directories(tmp_path):
    # mkdir(parents=True): the stage dir's own parents may not exist yet.
    binary, node_dist = "bin", "dist"
    root = _root(tmp_path, binary, node_dist)
    stage = tmp_path / "nested" / "missing" / "stage"

    def run(argv, cwd):
        return _Result(0)

    stage_hermetic_cli([], binary, node_dist, str(stage), root=str(root), run=run)
    assert (stage / "testing-conventions").read_bytes() == b"binary"


def test_stages_into_an_already_existing_stage_dir(tmp_path):
    # mkdir(exist_ok=True): a rerun (or a stage dir some earlier step already created) must
    # not raise FileExistsError.
    binary, node_dist = "bin", "dist"
    root = _root(tmp_path, binary, node_dist)
    stage = tmp_path / "stage"
    stage.mkdir(parents=True)

    def run(argv, cwd):
        return _Result(0)

    stage_hermetic_cli([], binary, node_dist, str(stage), root=str(root), run=run)
    assert (stage / "testing-conventions").read_bytes() == b"binary"


def test_stages_dist_over_an_already_existing_dist_dir(tmp_path):
    # copytree(dirs_exist_ok=True): a rerun (or a stage/dist an earlier step already created)
    # must merge rather than raise FileExistsError.
    binary, node_dist = "bin", "dist"
    root = _root(tmp_path, binary, node_dist)
    stage = tmp_path / "stage"
    (stage / "dist").mkdir(parents=True)

    def run(argv, cwd):
        return _Result(0)

    stage_hermetic_cli([], binary, node_dist, str(stage), root=str(root), run=run)
    assert (stage / "dist" / "mutation" / "main.js").read_text() == "adapter"


def test_raises_when_a_command_fails_and_stages_nothing(tmp_path):
    def run(argv, cwd):
        return _Result(2)

    try:
        stage_hermetic_cli([(["cargo", "build"], ".")], "bin", "dist", str(tmp_path / "stage"), root=str(tmp_path), run=run)
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "cargo build" in error.message
        assert "exited 2" in error.message
    else:
        raise AssertionError("a failing command must raise")
    assert not (tmp_path / "stage").exists()


def test_raises_when_a_command_is_killed_by_a_signal(tmp_path):
    # A negative returncode (POSIX signal death, e.g. OOM-killed cargo) must be treated as a
    # failure too — `!= 0`, not `> 0`, which a negative returncode would silently pass.
    def run(argv, cwd):
        return _Result(-9)

    try:
        stage_hermetic_cli([(["cargo", "build"], ".")], "bin", "dist", str(tmp_path / "stage"), root=str(tmp_path), run=run)
    except Exception as error:  # noqa: BLE001
        assert "exited -9" in error.message
    else:
        raise AssertionError("a signal-killed command must raise")
