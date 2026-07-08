"""End-to-end tests for the red-path-hermetic-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output — no `sys.argv` juggling,
no mocks.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.red_path_hermetic_wired.cli import cli
from checks.red_path_hermetic_wired.decide import RED_PATH_JOBS

REPO_ROOT = Path(__file__).resolve().parents[4]
WIRED_STEPS = (
    "    needs: [build-cli]\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "      - uses: ./.github/actions/download-hermetic-cli\n"
)
WIRED = "jobs:\n" + "".join(f"  {job}:\n{WIRED_STEPS}" for job in RED_PATH_JOBS)
# One job reverts to a bare npx run with no artifact download — the pre-#379 shape.
UNWIRED = WIRED.replace(
    "  coverage-rust-red:\n" + WIRED_STEPS,
    "  coverage-rust-red:\n    steps:\n      - run: npx -y testing-conventions unit coverage\n",
)


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "runs the hermetic CLI built from HEAD" in result.output


def test_fails_on_an_unwired_fixture(tmp_path):
    bad = tmp_path / "wf.yml"
    bad.write_text(UNWIRED)
    result = CliRunner().invoke(cli, [str(bad)])
    assert result.exit_code == 1
    assert "::error::" in result.output
    assert "coverage-rust-red" in result.output


def test_default_path_passes_against_the_real_selftest_workflow():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "runs the hermetic CLI built from HEAD" in result.output
