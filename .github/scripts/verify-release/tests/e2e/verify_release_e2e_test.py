"""End-to-end tests for verify_release's git-backed operations: real git, no mocks.

Per the standard, an e2e test runs with no mocks. Each test builds a real repo — committing the
two action-fetch targets and tagging published npm versions — then runs the script's `__main__`
entry in-process via `runpy` with the same argv the workflow passes, and asserts on the real exit
code and output. The `gh`-backed `dispatch-and-wait` path has no real external to run without
GitHub, so it stays integration-only; the archive/tag operations are exercised for real here.
"""
import runpy
import subprocess
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

SCRIPT = Path(__file__).resolve().parents[2] / "verify_release.py"


def _git(repo, *args):
    return subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True, text=True, check=True
    ).stdout.strip()


def _run_main(repo, argv, monkeypatch):
    """Run the script's __main__ in-process, from `repo`, with the given argv; return the exit code."""
    monkeypatch.chdir(repo)
    with patch.object(sys, "argv", ["verify_release.py", *argv]):
        try:
            runpy.run_path(str(SCRIPT), run_name="__main__")
        except SystemExit as exit_:
            return exit_.code
    return 0


@pytest.fixture
def repo(tmp_path):
    """A real repo carrying the two action-fetch targets and a couple of npm version tags."""
    r = tmp_path / "repo"
    r.mkdir()
    _git(r, "init", "-q")
    _git(r, "config", "user.email", "t@t.t")
    _git(r, "config", "user.name", "t")
    for rel in (".github/actions/detect/action.yml", "internals/detect/src/detect.py"):
        p = r / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text("x")
    _git(r, "add", "-A")
    _git(r, "commit", "-q", "-m", "release")
    _git(r, "tag", "testing-conventions-npm-v0.0.9")
    _git(r, "tag", "testing-conventions-npm-v0.0.67")
    _git(r, "tag", "testing-conventions-rust-v0.0.99")
    return r


def test_resolve_version_prints_the_numeric_max_npm_version(repo, monkeypatch, capsys):
    sha = _git(repo, "rev-parse", "HEAD")
    assert _run_main(repo, ["resolve-version", sha], monkeypatch) == 0
    assert capsys.readouterr().out.strip() == "0.0.67"


def test_check_layout_passes_when_both_action_targets_are_archived(repo, monkeypatch, capsys):
    sha = _git(repo, "rev-parse", "HEAD")
    assert _run_main(repo, ["check-layout", sha], monkeypatch) == 0
    assert "layout present" in capsys.readouterr().out


def test_check_layout_fails_closed_when_a_target_is_missing_from_the_archive(repo, monkeypatch, capsys):
    # Simulate the file-move/export-ignore regression: detect.py is gone at this commit.
    (repo / "internals/detect/src/detect.py").unlink()
    _git(repo, "add", "-A")
    _git(repo, "commit", "-q", "-m", "break the detect path")
    sha = _git(repo, "rev-parse", "HEAD")
    assert _run_main(repo, ["check-layout", sha], monkeypatch) == 1
    assert "internals/detect/src/detect.py" in capsys.readouterr().out
