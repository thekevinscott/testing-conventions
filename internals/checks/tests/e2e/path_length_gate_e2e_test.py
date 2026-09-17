"""End-to-end tests for the path-length-gate command: real git repos, click's CliRunner.

The command shells out to git in the working directory, so it runs here (the package-root e2e
suite), not the isolated unit suite. Each case builds a scratch repo, commits a realistic PR
shape, and invokes the command over that range — no fakes anywhere in the path.
"""
import os
import subprocess

from click.testing import CliRunner

from checks.path_length_gate.cli import cli

# One character over the 200-character budget path-length-gate holds added/renamed paths to.
OVERLONG_NAME = "a" * 205


def _git(repo, *args):
    subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True, text=True)


def _commit(repo, message):
    _git(repo, "add", "-A")
    _git(repo, "commit", "-m", message)
    return subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=repo, check=True, capture_output=True, text=True
    ).stdout.strip()


def _repo(tmp_path):
    _git(tmp_path, "init", "-q")
    _git(tmp_path, "config", "user.email", "test@example.com")
    _git(tmp_path, "config", "user.name", "Test")
    # The gate never signs anything; disabling it keeps the fixture independent of the
    # ambient `commit.gpgsign` a contributor's global config may set.
    _git(tmp_path, "config", "commit.gpgsign", "false")
    (tmp_path / "README.md").write_text("hello\n")
    return _commit(tmp_path, "initial commit")


def _write(path, text="content\n"):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def _gate(repo, base, head):
    old = os.getcwd()
    os.chdir(repo)
    try:
        return CliRunner().invoke(cli, [base, head])
    finally:
        os.chdir(old)


def test_a_short_added_path_passes(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / "notes" / "short.md")
    head = _commit(tmp_path, "add a short path")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 0
    assert "every added or renamed path fits the Windows checkout budget" in result.output


def test_an_overlong_added_path_fails_naming_the_path_length_and_budget(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / OVERLONG_NAME)
    head = _commit(tmp_path, "add an overlong path")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 1
    assert f"::error file={OVERLONG_NAME}::" in result.output
    assert "205" in result.output
    assert "200" in result.output


def test_a_renamed_path_landing_over_budget_fails(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / "short.md", "same content so the rename is detected\n")
    _commit(tmp_path, "add the short path")

    _git(tmp_path, "mv", "short.md", OVERLONG_NAME)
    head = _commit(tmp_path, "rename it over budget")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 1
    assert f"::error file={OVERLONG_NAME}::" in result.output


def test_editing_an_existing_overlong_path_without_renaming_it_does_not_trip(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / OVERLONG_NAME, "original\n")
    _commit(tmp_path, "a pre-existing overlong path")

    _write(tmp_path / OVERLONG_NAME, "edited\n")
    head = _commit(tmp_path, "edit the pre-existing overlong path")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 0
    assert "::error" not in result.output


def test_multiple_added_paths_names_only_the_offending_one(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / "notes" / "short.md")
    _write(tmp_path / OVERLONG_NAME)
    head = _commit(tmp_path, "add a short and an overlong path")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 1
    assert f"::error file={OVERLONG_NAME}::" in result.output
    assert "notes/short.md" not in result.output
