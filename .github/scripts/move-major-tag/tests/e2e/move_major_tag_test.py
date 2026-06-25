"""End-to-end tests for the move-major-tag script.

Per the standard, an e2e test runs with no mocks. git is the real external: each test builds a
work repo wired to a local bare `origin`, then runs `move_major_tag.py` as a subprocess exactly
as the workflow does (`SHA` / `TAG` in the env), and reads the moving tag back *from the remote*
to confirm the real fetch → decide → tag → push path.
"""
import os
import subprocess
import sys
from pathlib import Path

import pytest

SCRIPT = Path(__file__).resolve().parents[2] / "move_major_tag.py"


def _git(repo, *args):
    """Run a git command in `repo`, returning trimmed stdout (raises on failure)."""
    return subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True, text=True, check=True,
    ).stdout.strip()


def _commit(repo, message):
    """Create an empty commit in `repo` and return its full sha."""
    _git(repo, "commit", "--allow-empty", "-q", "-m", message)
    return _git(repo, "rev-parse", "HEAD")


def _run_script(repo, sha, tag="v0"):
    """Run move_major_tag.py against `repo`, the way the workflow invokes it."""
    env = {**os.environ, "SHA": sha, "TAG": tag}
    return subprocess.run(
        [sys.executable, str(SCRIPT)],
        cwd=repo, env=env, capture_output=True, text=True,
    )


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


def _remote_tag(origin, tag="v0"):
    """The sha the tag points at on the remote, or '' if the tag is absent there."""
    out = subprocess.run(
        ["git", "-C", str(origin), "rev-parse", "-q", "--verify", f"refs/tags/{tag}"],
        capture_output=True, text=True,
    )
    return out.stdout.strip() if out.returncode == 0 else ""


def test_e2e_bootstraps_the_tag_when_absent(repo):
    work, origin, first = repo
    result = _run_script(work, first)
    assert result.returncode == 0, result.stderr
    assert _remote_tag(origin) == first  # created + pushed to origin
    assert "bootstrap" in result.stdout.lower()


def test_e2e_advances_the_tag_forward(repo):
    work, origin, first = repo
    _run_script(work, first)  # bootstrap at first
    second = _commit(work, "second")
    _git(work, "push", "-q", "origin", "main")
    result = _run_script(work, second)
    assert result.returncode == 0, result.stderr
    assert _remote_tag(origin) == second  # advanced forward on the remote


def test_e2e_does_not_rewind_to_an_older_commit(repo):
    work, origin, first = repo
    second = _commit(work, "second")
    _git(work, "push", "-q", "origin", "main")
    _run_script(work, second)            # tag at the newer commit
    assert _remote_tag(origin) == second
    result = _run_script(work, first)    # an out-of-order older release lands
    assert result.returncode == 0, result.stderr
    assert _remote_tag(origin) == second  # unchanged: never rewound
    assert "nothing to do" in result.stdout.lower()


def test_e2e_reapplying_the_current_commit_is_a_noop(repo):
    work, origin, first = repo
    _run_script(work, first)
    result = _run_script(work, first)  # same sha again (a re-run)
    assert result.returncode == 0, result.stderr
    assert _remote_tag(origin) == first
    assert "nothing to do" in result.stdout.lower()


def test_e2e_missing_sha_fails_clearly(repo):
    work, origin, _first = repo
    result = _run_script(work, "")  # empty SHA
    assert result.returncode == 1
    assert "SHA is required" in (result.stdout + result.stderr)
    assert _remote_tag(origin) == ""  # nothing pushed
