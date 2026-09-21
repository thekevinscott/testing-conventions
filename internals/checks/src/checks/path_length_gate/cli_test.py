"""Colocated unit tests for the path-length-gate command."""
from checks.path_length_gate.cli import cli


def _patch_run(monkeypatch, verdict):
    """Stand a recording fake in for the gate; return the list of roots it was called with."""
    seen = []

    def run(root):
        seen.append(root)
        return verdict

    monkeypatch.setattr("checks.path_length_gate.cli.run", run)
    return seen


def _exit_code(root="."):
    """Drive the callback and return the exit code it raised."""
    try:
        cli.callback(root=root)
    except SystemExit as exit_:
        return exit_.code
    raise AssertionError("the command must exit through SystemExit")


def test_declares_the_root_argument():
    (root,) = cli.params
    assert root.name == "root"
    assert root.required is True


def test_threads_the_root_into_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _exit_code(root="pkg")
    assert seen == ["pkg"]


def test_exits_zero_when_the_gate_holds(monkeypatch):
    _patch_run(monkeypatch, 0)
    assert _exit_code() == 0


def test_exits_nonzero_when_the_gate_reports_a_violation(monkeypatch):
    _patch_run(monkeypatch, 1)
    assert _exit_code() == 1
