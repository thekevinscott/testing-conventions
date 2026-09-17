"""End-to-end tests for the path-length-gate command: real git repos, click's CliRunner.

The command shells out to git against a working directory, so it runs here (the package-root e2e
suite), not the isolated unit suite. Each case builds a scratch repo and invokes the command over
it — no fakes anywhere in the path.
"""
import subprocess

from click.testing import CliRunner

from checks.path_length_gate.cli import cli

# One character over the 200-character budget path-length-gate holds every tracked path to.
OVERLONG_NAME = "a" * 205


def _git(repo, *args):
    subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True, text=True)


def _commit(repo, message):
    _git(repo, "add", "-A")
    _git(repo, "commit", "-m", message)


def _repo(tmp_path):
    _git(tmp_path, "init", "-q")
    _git(tmp_path, "config", "user.email", "test@example.com")
    _git(tmp_path, "config", "user.name", "Test")
    # The gate never signs anything; disabling it keeps the fixture independent of the
    # ambient `commit.gpgsign` a contributor's global config may set.
    _git(tmp_path, "config", "commit.gpgsign", "false")


def _write(path, text="content\n"):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def _gate(repo):
    return CliRunner().invoke(cli, [str(repo)])


def test_a_tree_of_only_short_paths_passes(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "notes" / "short.md")
    _commit(tmp_path, "add a short path")

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "every tracked path fits the Windows checkout budget" in result.output


def test_a_tracked_path_at_exactly_the_budget_passes(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / ("a" * 200))
    _commit(tmp_path, "add a path at the budget")

    result = _gate(tmp_path)
    assert result.exit_code == 0


def test_an_overlong_tracked_path_fails_naming_the_path_length_and_budget(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / OVERLONG_NAME)
    _commit(tmp_path, "add an overlong path")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert f"::error file={OVERLONG_NAME}::" in result.output
    assert "205" in result.output
    assert "200" in result.output


def test_multiple_overlong_paths_each_get_an_annotation(tmp_path):
    other_overlong = "b" * 210
    _repo(tmp_path)
    _write(tmp_path / OVERLONG_NAME)
    _write(tmp_path / other_overlong)
    _commit(tmp_path, "add two overlong paths")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert f"::error file={OVERLONG_NAME}::" in result.output
    assert f"::error file={other_overlong}::" in result.output


def test_an_untracked_overlong_path_does_not_trip_the_gate(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "notes" / "short.md")
    _commit(tmp_path, "add a short path")
    _write(tmp_path / OVERLONG_NAME)

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "::error" not in result.output
