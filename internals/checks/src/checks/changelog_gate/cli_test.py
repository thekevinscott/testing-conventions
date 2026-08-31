"""Colocated unit tests for the changelog-gate command (isolation — the gate is patched out).

The command is one delegation: thread the two SHAs into `gate.run`, then turn its verdict into
the process's exit code. Both halves are asserted here against a `run` patched by string, so no
collaborator is imported and no git subprocess is spawned. The decisions are covered in
`decide_test.py`, the orchestration in `gate_test.py`, and the real git path in the e2e suite.
"""
from checks.changelog_gate.cli import cli


def _patch_run(monkeypatch, verdict):
    """Stand a recording fake in for the gate; return the list of (base, head) it was called with."""
    seen = []

    def run(base_sha, head_sha):
        seen.append((base_sha, head_sha))
        return verdict

    monkeypatch.setattr("checks.changelog_gate.cli.run", run)
    return seen


def _exit_code():
    """Drive the callback and return the exit code it raised."""
    try:
        cli.callback(base_sha="abc", head_sha="def")
    except SystemExit as exit_:
        return exit_.code
    raise AssertionError("the command must exit through SystemExit")


def test_declares_the_base_and_head_sha_arguments():
    # Assert click's own registered metadata — `.callback` bypasses arg parsing, so this is what
    # pins the `@click.argument` decorators without a CliRunner collaborator.
    base, head = cli.params
    assert base.name == "base_sha"
    assert head.name == "head_sha"
    assert (base.required, head.required) == (True, True)


def test_threads_the_shas_into_the_gate_base_first(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _exit_code()
    assert seen == [("abc", "def")]


def test_exits_zero_when_the_gate_holds(monkeypatch):
    _patch_run(monkeypatch, 0)
    assert _exit_code() == 0


def test_exits_nonzero_when_the_gate_reports_a_finding(monkeypatch):
    _patch_run(monkeypatch, 1)
    assert _exit_code() == 1
