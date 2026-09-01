"""Colocated unit tests for the pnpm-version-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path
is asserted against the propagated exception's `.message` rather than importing `CheckFailed`.
"""
from checks.pnpm_version_wired.cli import (
    DERIVED,
    FALLBACK,
    REUSABLE_WORKFLOW,
    cli,
    setup_versions,
)

def step(version: str) -> str:
    return f"      - uses: pnpm/action-setup@v5\n        with:\n          version: {version}\n"


GUARDED = "${{ " + DERIVED + " " + FALLBACK + " }}"
WIRED_STEP = step(GUARDED)
LITERAL_STEP = step('">=11"')
# Reads detect with no fallback, so a detect predating the output leaves `version` empty.
UNGUARDED_STEP = step("${{ " + DERIVED + " }}")


def test_setup_versions_reads_each_steps_version_in_order():
    assert setup_versions(LITERAL_STEP + WIRED_STEP) == ['">=11"', GUARDED]


def test_setup_versions_ignores_a_version_belonging_to_a_later_step():
    # A pnpm step that sets no version must contribute nothing rather than borrow the next
    # step's — otherwise an unpinned step reads as wired.
    text = "      - uses: pnpm/action-setup@v5\n      - uses: actions/setup-node@v6\n        with:\n          version: 24\n"
    assert setup_versions(text) == []


def test_setup_versions_finds_nothing_without_a_pnpm_step():
    assert setup_versions("      - uses: actions/checkout@v6\n") == []


def test_setup_versions_reads_a_step_whose_uses_follows_its_if_line():
    # The real steps open with `- if:` and carry `uses:` a line later, so the step chunk — not
    # the `uses:` line — is what has to be searched.
    text = (
        "      - if: matrix.language == 'typescript'\n"
        "        uses: pnpm/action-setup@v5\n"
        "        with:\n"
        f"          version: {GUARDED}\n"
    )
    assert setup_versions(text) == [GUARDED]


def test_setup_versions_ignores_a_version_declared_before_any_step():
    # The workflow declares its own `version:` input far above the jobs. Lines preceding the
    # first step belong to no step, so they must not be read as one step's pin.
    text = "on:\n  workflow_call:\n    inputs:\n      version:\n        type: string\n"
    assert setup_versions(text) == []


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
