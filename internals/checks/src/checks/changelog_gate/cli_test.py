"""Colocated unit tests for the changelog-gate command (isolation — no CliRunner).

The command is driven through its `.callback` against a real scratch git repo, so the wiring it
owns — the SHAs it threads into the git reads, and the exit code it turns the verdict into — is
exercised for real rather than against a fake. The decisions themselves are covered in
`decide_test.py` and the orchestration in `gate_test.py`.
"""
import os
import subprocess

from checks.changelog_gate.cli import cli


def _git(repo, *args):
    subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True, text=True)


def _commit(repo, message):
    _git(repo, "add", "-A")
    # Signing is disabled repo-locally in `_repo`, so this never reaches a signing agent.
    _git(repo, "commit", "-m", message)
    return subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=repo, check=True, capture_output=True, text=True
    ).stdout.strip()


def _repo(tmp_path):
    """A scratch repo with one commit, returned with that commit's SHA as the base."""
    _git(tmp_path, "init", "-q")
    _git(tmp_path, "config", "user.email", "test@example.com")
    _git(tmp_path, "config", "user.name", "Test")
    _git(tmp_path, "config", "commit.gpgsign", "false")
    (tmp_path / "README.md").write_text("hello\n")
    return _commit(tmp_path, "initial commit")


def _write(path, text="// code\n"):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def _drive(repo, base_sha, head_sha):
    """Run the callback with `repo` as the working directory; return its exit code."""
    old = os.getcwd()
    os.chdir(repo)
    try:
        cli.callback(base_sha=base_sha, head_sha=head_sha)
    except SystemExit as exit_:
        return exit_.code
    finally:
        os.chdir(old)
    raise AssertionError("the command must exit through SystemExit")


def test_declares_the_base_and_head_sha_arguments():
    # Assert click's own registered metadata — `.callback` bypasses arg parsing, so this is what
    # pins the `@click.argument` decorators without a CliRunner collaborator.
    base, head = cli.params
    assert base.name == "base_sha"
    assert head.name == "head_sha"
    assert (base.required, head.required) == (True, True)


def test_exits_nonzero_when_package_source_changes_without_fragments(tmp_path, capsys):
    base = _repo(tmp_path)
    _write(tmp_path / "packages" / "rust" / "src" / "lib.rs")
    head = _commit(tmp_path, "add rust code")

    assert _drive(tmp_path, base, head) == 1
    assert "packages/rust has code changes" in capsys.readouterr().out


def test_exits_zero_when_both_fragments_are_added(tmp_path, capsys):
    base = _repo(tmp_path)
    _write(tmp_path / "packages" / "rust" / "src" / "lib.rs")
    _write(tmp_path / "packages" / "rust" / "changelog.d" / "2026-08-30-a-fix.md", "**Fixed** it.\n")
    _write(tmp_path / "packages" / "rust" / "migrations.d" / "2026-08-30-a-fix.md", "### It\n")
    head = _commit(tmp_path, "add rust code with fragments")

    assert _drive(tmp_path, base, head) == 0
    assert "::error::" not in capsys.readouterr().out
