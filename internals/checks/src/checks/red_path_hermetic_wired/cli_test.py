"""Colocated unit tests for the red-path-hermetic-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path
is asserted against the propagated exception's `.message` rather than importing `CheckFailed`; the
pure decision (and the `RED_PATH_JOBS` list it iterates) has its own `decide_test.py`.
"""
from checks.red_path_hermetic_wired.cli import SELFTEST_WORKFLOW, cli

# Every red-path job wired: a `needs: [build-cli]` edge and the download step. Written inline (not
# built from decide's `RED_PATH_JOBS`) so this unit test imports no collaborator.
WIRED = """\
jobs:
  below-floor:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
  mutation-gate:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
  python-mutation-clean:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
  isolation-red:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
  packaging-red:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
  coverage-rust-red:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
  integration-lint-new-arms-trip:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
  packaging-package-root-red:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
  colocated-rust-red:
    needs: [build-cli]
    steps:
      - uses: ./.github/actions/download-hermetic-cli
"""

# `packaging-red` reverts to a bare npx run with no artifact download — the pre-#379 shape.
UNWIRED = WIRED.replace(
    "  packaging-red:\n    needs: [build-cli]\n    steps:\n      - uses: ./.github/actions/download-hermetic-cli\n",
    "  packaging-red:\n    steps:\n      - run: npx -y testing-conventions packaging\n",
)


def test_declares_the_workflow_argument_defaulting_to_the_selftest_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == SELFTEST_WORKFLOW


def test_echoes_when_every_red_path_job_is_wired(tmp_path, capsys):
    wf = tmp_path / "wf.yml"
    wf.write_text(WIRED)
    cli.callback(workflow=str(wf))
    assert "runs the hermetic CLI built from HEAD" in capsys.readouterr().out


def test_raises_naming_an_unwired_job(tmp_path):
    wf = tmp_path / "wf.yml"
    wf.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(wf))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "packaging-red" in error.message
    else:
        raise AssertionError("an unwired red-path job must raise")
