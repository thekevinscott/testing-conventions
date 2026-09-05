"""Colocated unit tests for the pnpm-version-wired command (isolation — no `CliRunner`).

The chunking and version reads are pinned in `pnpm_steps_test.py` / `setup_versions_test.py`;
here the `cli` command is driven through its `.callback` (the undecorated function), and the
raise path is asserted against the propagated exception's `.message`.
"""
from checks.pnpm_version_wired.cli import DERIVED, FALLBACK, REUSABLE_WORKFLOW, cli


def step(version: str) -> str:
    return f"      - uses: pnpm/action-setup@v5\n        with:\n          version: {version}\n"


GUARDED = "${{ " + DERIVED + " " + FALLBACK + " }}"
WIRED_STEP = step(GUARDED)
LITERAL_STEP = step('">=11"')
# Reads detect with no fallback, so a detect predating the output leaves `version` empty.
UNGUARDED_STEP = step("${{ " + DERIVED + " }}")


def test_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED_STEP)
    cli.callback(workflow=str(workflow))
    assert "take their version from detect" in capsys.readouterr().out


def test_raises_on_a_literal_version(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(LITERAL_STEP)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "pin a literal pnpm version" in error.message
    else:
        raise AssertionError("a literal pnpm version must raise")


def test_raises_when_no_step_sets_a_version(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("      - uses: pnpm/action-setup@v5\n")
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "No pnpm version is specified" in error.message
    else:
        raise AssertionError("a workflow that pins nothing at all must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_raises_on_a_derived_version_with_no_stale_detect_fallback(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNGUARDED_STEP)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "No pnpm version is specified" in error.message
    else:
        raise AssertionError("a derived version with no fallback must raise")


def test_echoes_the_fallback_in_the_success_line(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED_STEP)
    cli.callback(workflow=str(workflow))
    assert "stale-detect fallback" in capsys.readouterr().out
