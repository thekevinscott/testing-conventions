"""End-to-end tests for the changelog-gate command: real git repos, click's CliRunner.

The command shells out to git in the working directory, so it runs here (the package-root e2e
suite), not the isolated unit suite. Each case builds a scratch repo, commits a realistic PR
shape, and invokes the command over that range — no fakes anywhere in the path.
"""
import os
import subprocess

from click.testing import CliRunner

from checks.changelog_gate.cli import cli


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


def _write(path, text="// code\n"):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def _gate(repo, base, head):
    old = os.getcwd()
    os.chdir(repo)
    try:
        return CliRunner().invoke(cli, [base, head])
    finally:
        os.chdir(old)


def test_package_source_without_fragments_fails(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / "packages" / "rust" / "src" / "lib.rs")
    head = _commit(tmp_path, "add rust code")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 1
    assert "packages/rust has code changes" in result.output
    assert "packages/rust/changelog.d/YYYY-MM-DD-<slug>.md" in result.output


def test_package_source_with_both_fragments_passes(tmp_path):
    base = _repo(tmp_path)
    rust = tmp_path / "packages" / "rust"
    _write(rust / "src" / "lib.rs")
    _write(rust / "changelog.d" / "2026-08-30-a-fix.md", "**Fixed** a thing.\n")
    _write(rust / "migrations.d" / "2026-08-30-a-fix.md", "### A thing\n")
    head = _commit(tmp_path, "add rust code with fragments")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 0
    assert "changelog and migrations fragments present" in result.output


def test_two_packages_each_need_their_own_fragments(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / "packages" / "rust" / "src" / "lib.rs")
    _write(tmp_path / "packages" / "rust" / "changelog.d" / "2026-08-30-a-fix.md", "**Fixed**\n")
    _write(tmp_path / "packages" / "rust" / "migrations.d" / "2026-08-30-a-fix.md", "### x\n")
    _write(tmp_path / "packages" / "node" / "src" / "index.ts")
    head = _commit(tmp_path, "touch two packages, fragment only one")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 1
    assert "packages/node has code changes" in result.output
    assert "packages/rust has code changes" not in result.output


def test_the_skip_line_bypasses_the_gate(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / "packages" / "rust" / "src" / "lib.rs")
    head = _commit(tmp_path, "internal rename\n\nskip-changelog: pure rename, no observable change")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 0
    assert "bypassing changelog enforcement" in result.output


def test_editing_only_the_frozen_stubs_needs_no_fragment(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / "packages" / "rust" / "CHANGELOG.md", "# Changelog\n")
    head = _commit(tmp_path, "touch the frozen archive")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 0


def test_a_malformed_fragment_name_fails(tmp_path):
    base = _repo(tmp_path)
    rust = tmp_path / "packages" / "rust"
    _write(rust / "src" / "lib.rs")
    _write(rust / "changelog.d" / "fix-it.md", "**Fixed** a thing.\n")
    _write(rust / "migrations.d" / "2026-08-30-a-fix.md", "### A thing\n")
    head = _commit(tmp_path, "misnamed fragment")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 1
    assert "::error file=packages/rust/changelog.d/fix-it.md::" in result.output


def test_a_pr_that_touches_no_package_passes(tmp_path):
    base = _repo(tmp_path)
    _write(tmp_path / "internals" / "checks" / "src" / "checks" / "cli.py", "# code\n")
    head = _commit(tmp_path, "internal tooling only")

    result = _gate(tmp_path, base, head)
    assert result.exit_code == 0
    assert "No package source changed" in result.output
