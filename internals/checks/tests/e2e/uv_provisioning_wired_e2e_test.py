"""End-to-end tests for the uv-provisioning-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output — no `sys.argv` juggling,
no mocks.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.uv_provisioning_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
PYTHON_ARM = (
    "      - if: matrix.language == 'python'\n"
    "        uses: astral-sh/setup-uv@v7\n"
    "      - if: matrix.language == 'python'\n"
    "        name: Provision the Python suite environment (uv)\n"
    "        run: uv sync\n"
)
WIRED = (
    "jobs:\n"
    f"  unit-coverage:\n    steps:\n{PYTHON_ARM}"
    f"  coverage-changed:\n    steps:\n{PYTHON_ARM}"
    f"  mutation:\n    steps:\n{PYTHON_ARM}"
    "  integration-lint:\n    steps:\n      - run: echo lint\n"
)
# A pip arm survives beside the uv arm — the dual-provisioning shape #399 removes.
UNWIRED = WIRED + (
    "      - uses: actions/setup-python@v6\n"
    "      - run: python -m pip install --quiet coverage pytest\n"
)


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "provision Python with uv alone" in result.output


def test_fails_on_a_dual_provisioning_fixture(tmp_path):
    bad = tmp_path / "wf.yml"
    bad.write_text(UNWIRED)
    result = CliRunner().invoke(cli, [str(bad)])
    assert result.exit_code == 1
    assert "::error::" in result.output


def test_default_path_passes_against_the_real_workflow():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "provision Python with uv alone" in result.output
