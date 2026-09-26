"""End-to-end tests for the agents-md-size command: real git repos, real files, click's CliRunner.

The command shells out to git and reads the working tree, so it runs here (the package-root e2e
suite), not the isolated unit suite. Each case builds a scratch repo and invokes the command over
it — no fakes anywhere in the path.
"""
import subprocess

from click.testing import CliRunner

from checks.agents_md_size.gate import MAX_CHARS, cli


def _git(repo, *args):
    subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True, text=True)


def _commit(repo, message):
    _git(repo, "add", "-A")
    _git(repo, "commit", "-m", message)


def _repo(tmp_path):
    _git(tmp_path, "init", "-q", "-b", "main")
    _git(tmp_path, "config", "user.email", "test@example.com")
    _git(tmp_path, "config", "user.name", "Test")
    # The gate never signs anything; disabling it keeps the fixture independent of the
    # ambient `commit.gpgsign` a contributor's global config may set.
    _git(tmp_path, "config", "commit.gpgsign", "false")


def _write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def _document(chars):
    return "x" * chars


def _gate(repo, *args):
    return CliRunner().invoke(cli, [str(repo), *args])


def test_a_short_instructions_file_passes(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", _document(100))
    _commit(tmp_path, "add a short AGENTS.md")

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "every instructions file fits its budget" in result.output


def test_a_file_at_exactly_the_budget_does_not_fail(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", _document(MAX_CHARS))
    _commit(tmp_path, "add an AGENTS.md at the budget")

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "::error" not in result.output


def test_a_file_one_character_over_the_budget_fails(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", _document(MAX_CHARS + 1))
    _commit(tmp_path, "add an over-budget AGENTS.md")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert "::error file=AGENTS.md::" in result.output
    # The literal budget, not `MAX_CHARS`: an assertion that reads the constant moves with it.
    assert "30001 characters" in result.output
    assert "30000-character budget" in result.output


def test_a_nested_claude_md_is_held_to_the_same_budget(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "pkg" / "CLAUDE.md", _document(MAX_CHARS + 1))
    _commit(tmp_path, "add an over-budget nested CLAUDE.md")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert "::error file=pkg/CLAUDE.md::" in result.output


def test_an_imported_file_counts_toward_the_importer_budget(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", "@agents/style.md\n")
    _write(tmp_path / "agents" / "style.md", _document(MAX_CHARS))
    _commit(tmp_path, "add a short index importing a long file")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert "::error file=AGENTS.md::" in result.output


def test_an_import_inside_a_fenced_block_is_not_resolved(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", "```\n@agents/style.md\n```\n")
    _write(tmp_path / "agents" / "style.md", _document(MAX_CHARS + 1))
    _commit(tmp_path, "add a fenced import")

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "::error" not in result.output


def test_an_untracked_over_budget_file_does_not_trip_the_gate(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", _document(100))
    _commit(tmp_path, "add a short AGENTS.md")
    _write(tmp_path / "pkg" / "AGENTS.md", _document(MAX_CHARS + 1))

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "::error" not in result.output


def test_a_raised_budget_lets_an_otherwise_failing_file_pass(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", _document(MAX_CHARS + 1))
    _commit(tmp_path, "add an over-budget AGENTS.md")

    result = _gate(tmp_path, "--max-chars", str(MAX_CHARS + 1))
    assert result.exit_code == 0
    assert "::error" not in result.output
