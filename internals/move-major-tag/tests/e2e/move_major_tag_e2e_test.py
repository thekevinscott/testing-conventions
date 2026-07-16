"""End-to-end tests for the move-major-tag helper: real git, no mocks.

Per the standard, an e2e test runs with no mocks. Each test builds a work repo wired to a local
bare `origin`, then runs the script's `__main__` entry point in-process via `runpy` — with
`SHA` / `TAG` set the way the workflow's env does — and reads the moving tag back *from the
remote*. Running the real entry point in-process keeps the whole fetch -> decide -> tag -> push
path (and the `__main__` guard) on the measured-coverage path; the env is set with
`patch.dict` and the working directory is confined to the fixture, never the test body.
"""
import os
import runpy
import subprocess
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

SCRIPT = Path(__file__).resolve().parents[2] / "src" / "move_major_tag.py"


def _git(repo, *args):
    return subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True, text=True, check=True
    ).stdout.strip()


def _commit(repo, message):
    _git(repo, "commit", "--allow-empty", "-q", "-m", message)
    return _git(repo, "rev-parse", "HEAD")


def _remote_tag(origin, tag="v0"):
    out = subprocess.run(
        ["git", "-C", str(origin), "rev-parse", "-q", "--verify", f"refs/tags/{tag}"],
        capture_output=True, text=True,
    )
    return out.stdout.strip() if out.returncode == 0 else ""


@pytest.fixture
def repo(tmp_path):
    """A work repo on `main` wired to a local bare `origin`, with one pushed commit."""
    origin = tmp_path / "origin.git"
    subprocess.run(["git", "init", "-q", "--bare", str(origin)], check=True)
    work = tmp_path / "work"
    subprocess.run(["git", "init", "-q", "-b", "main", str(work)], check=True)
    _git(work, "config", "user.email", "ci@example.com")
    _git(work, "config", "user.name", "CI")
    _git(work, "remote", "add", "origin", str(origin))
    first = _commit(work, "first")
    _git(work, "push", "-q", "origin", "main")
    return work, origin, first


@pytest.fixture
def run_in_repo(repo):
    """A `run(sha, tag='v0') -> exit_code` that runs the script as `__main__` inside the work
    repo, with the env set via `patch.dict`. Yields it with the repo handles; the chdir is
    confined to this fixture so the helper's cwd-relative git resolves against the temp repo.
    """
    work, origin, first = repo
    origin_cwd = os.getcwd()
    os.chdir(work)

    def run(sha, tag="v0"):
        with patch.dict(os.environ, {"SHA": sha, "TAG": tag}):
            try:
                runpy.run_path(str(SCRIPT), run_name="__main__")
                return 0
            except SystemExit as exit_:
                return exit_.code or 0

    try:
        yield run, origin, first
    finally:
        os.chdir(origin_cwd)


def test_e2e_bootstraps_the_tag_when_absent(run_in_repo, capsys):
    run, origin, first = run_in_repo
    assert run(first) == 0
    assert _remote_tag(origin) == first
    assert "bootstrap" in capsys.readouterr().out.lower()


def test_e2e_advances_the_tag_forward(run_in_repo, capsys):
    run, origin, first = run_in_repo
    run(first)
    second = _commit(Path.cwd(), "second")
    _git(Path.cwd(), "push", "-q", "origin", "main")
    assert run(second) == 0
    assert _remote_tag(origin) == second
    assert "advanced" in capsys.readouterr().out.lower()


def test_e2e_does_not_rewind_to_an_older_commit(run_in_repo, capsys):
    run, origin, first = run_in_repo
    second = _commit(Path.cwd(), "second")
    _git(Path.cwd(), "push", "-q", "origin", "main")
    run(second)
    assert _remote_tag(origin) == second
    assert run(first) == 0                 # an out-of-order older release lands
    assert _remote_tag(origin) == second   # unchanged: never rewound
    assert "nothing to do" in capsys.readouterr().out.lower()


def test_e2e_missing_sha_fails_clearly(run_in_repo, capsys):
    run, origin, _first = run_in_repo
    assert run("") == 1
    assert "SHA is required" in capsys.readouterr().out
    assert _remote_tag(origin) == ""  # nothing pushed


def test_e2e_blank_tag_falls_back_to_v0(run_in_repo):
    run, origin, first = run_in_repo
    assert run(first, tag="") == 0  # empty TAG -> default v0
    assert _remote_tag(origin, "v0") == first
