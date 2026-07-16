"""End-to-end tests for the engines-drift command: real files, click's CliRunner."""
from pathlib import Path

from click.testing import CliRunner

from checks.engines_drift.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
REAL_LOCK = REPO_ROOT / ".github" / "uv" / "engines.txt"


def test_passes_when_latest_equals_the_committed_lock(tmp_path):
    # The committed lock compared against itself is the no-drift case, over the real file.
    result = CliRunner().invoke(cli, ["--latest", str(REAL_LOCK), "--lock", str(REAL_LOCK)])
    assert result.exit_code == 0
    assert "no drift" in result.output


def test_fails_and_names_the_drift_when_latest_moved(tmp_path):
    lock = tmp_path / "lock.txt"
    lock.write_text("pytest==9.1.1\n")
    latest = tmp_path / "latest.txt"
    latest.write_text("pytest==9.9.9\n")
    result = CliRunner().invoke(cli, ["--latest", str(latest), "--lock", str(lock)])
    assert result.exit_code == 1
    assert "::error::" in result.output
    assert "pytest: 9.1.1 → 9.9.9" in result.output
