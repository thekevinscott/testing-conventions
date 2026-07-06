"""End-to-end tests for the e2e-verify-scope-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output — no `sys.argv` juggling,
no mocks. A no-arg run from the repo root exercises the default-path branch against the real
`.github/workflows/testing-conventions.yml`, which must be fully wired.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.e2e_verify_scope_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
WIRED = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    steps:
      - name: Verify the e2e attestation is current
        env:
          SCAN_PATH: ${{ inputs.path }}
          BASE: ${{ inputs.base }}
          EXTRA_SCOPE: ${{ needs.detect.outputs.e2e_extra_scope }}
          EXCLUDE: ${{ needs.detect.outputs.e2e_exclude }}
        run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE" $EXTRA_SCOPE $EXCLUDE

  packaging:
    name: Packaging
"""
UNWIRED = """\
  e2e-verify:
    steps:
      - run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT"

  packaging:
    name: Packaging
"""


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "appends detect's extra-scope/exclude roots" in result.output


def test_fails_on_a_broken_fixture(tmp_path):
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
    assert "appends detect's extra-scope/exclude roots" in result.output
