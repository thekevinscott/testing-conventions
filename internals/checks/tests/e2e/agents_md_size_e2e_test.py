"""End-to-end tests for the agents-md-size command: real git repos, real files, click's CliRunner.

The command shells out to git and reads the files it finds, so it runs here (the package-root e2e
suite), not the isolated unit suite. Each case builds a scratch repo and invokes the command over
it — no fakes anywhere in the path.
"""
import subprocess

from click.testing import CliRunner

from checks.agents_md_size.cli import cli

# Small enough to keep the fixtures readable; the real default is 16384.
BUDGET = 100


def _git(repo, *args):
    subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True, text=True)


def _repo(tmp_path):
    _git(tmp_path, "init", "-q")
    _git(tmp_path, "config", "user.email", "test@example.com")
    _git(tmp_path, "config", "user.name", "Test")
    # The gate never signs anything; disabling it keeps the fixture independent of the
    # ambient `commit.gpgsign` a contributor's global config may set.
    _git(tmp_path, "config", "commit.gpgsign", "false")


def _write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def _commit(repo, message):
    _git(repo, "add", "-A")
    _git(repo, "commit", "-m", message)


def _gate(repo, budget=BUDGET):
    return CliRunner().invoke(cli, [str(repo), "--budget", str(budget)])


def test_a_short_agents_md_passes(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", "# Rules\n\nBe brief.\n")
    _commit(tmp_path, "add a short AGENTS.md")

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "every instructions file fits the budget" in result.output


def test_an_over_budget_agents_md_fails_naming_the_budget_and_the_remedy(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", "x" * (BUDGET + 1))
    _commit(tmp_path, "add an over-budget AGENTS.md")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert "::error file=AGENTS.md::" in result.output
    assert str(BUDGET) in result.output
    assert "reference files or skills" in result.output


def test_a_nested_claude_md_is_held_to_the_same_budget(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "packages" / "web" / "CLAUDE.md", "y" * (BUDGET + 1))
    _commit(tmp_path, "add a nested over-budget CLAUDE.md")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert "::error file=packages/web/CLAUDE.md::" in result.output


def test_an_ordinary_markdown_file_is_not_measured(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "README.md", "z" * (BUDGET * 5))
    _commit(tmp_path, "add a long README")

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "::error" not in result.output


def test_an_import_counts_against_the_importing_file(tmp_path):
    # The whole point of counting after resolution: a short index importing a long file costs
    # the same context as one long file, and a naive byte count would pass it.
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", "@rules/detail.md\n")
    _write(tmp_path / "rules" / "detail.md", "d" * (BUDGET + 1))
    _commit(tmp_path, "add an index that imports a long file")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert "::error file=AGENTS.md::" in result.output


def test_an_untracked_over_budget_agents_md_does_not_trip_the_gate(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "README.md", "ok\n")
    _commit(tmp_path, "add a README")
    _write(tmp_path / "AGENTS.md", "x" * (BUDGET + 1))

    result = _gate(tmp_path)
    assert result.exit_code == 0
    assert "::error" not in result.output


def test_two_over_budget_files_each_get_an_annotation(tmp_path):
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", "x" * (BUDGET + 1))
    _write(tmp_path / "pkg" / "CLAUDE.md", "y" * (BUDGET + 1))
    _commit(tmp_path, "add two over-budget instructions files")

    result = _gate(tmp_path)
    assert result.exit_code == 1
    assert "::error file=AGENTS.md::" in result.output
    assert "::error file=pkg/CLAUDE.md::" in result.output


def test_the_default_budget_passes_this_repos_own_fixture(tmp_path):
    # No `--budget`: the shipped default has to be reachable from the command line.
    _repo(tmp_path)
    _write(tmp_path / "AGENTS.md", "# Rules\n\nBe brief.\n")
    _commit(tmp_path, "add a short AGENTS.md")

    result = CliRunner().invoke(cli, [str(tmp_path)])
    assert result.exit_code == 0
