"""Colocated unit tests for the red-path-hermetic-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path
is asserted against the propagated exception's `.message` rather than importing `CheckFailed`.
"""
from checks.red_path_hermetic_wired.cli import SELFTEST_WORKFLOW, cli
from checks.red_path_hermetic_wired.decide import RED_PATH_JOBS

WIRED_STEPS = (
    "    needs: [build-cli]\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "      - uses: ./.github/actions/download-hermetic-cli\n"
)
WIRED = "jobs:\n" + "".join(f"  {job}:\n{WIRED_STEPS}" for job in RED_PATH_JOBS)


def test_echoes_when_every_red_path_job_is_wired(tmp_path, capsys):
    wf = tmp_path / "wf.yml"
    wf.write_text(WIRED)
    cli.callback(workflow=str(wf))
    assert "runs the hermetic CLI built from HEAD" in capsys.readouterr().out


def test_raises_naming_an_unwired_job(tmp_path):
    unwired = WIRED.replace(
        "  packaging-red:\n" + WIRED_STEPS,
        "  packaging-red:\n    steps:\n      - run: npx -y testing-conventions packaging\n",
    )
    wf = tmp_path / "wf.yml"
    wf.write_text(unwired)
    try:
        cli.callback(workflow=str(wf))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "packaging-red" in error.message
    else:
        raise AssertionError("an unwired red-path job must raise")


def test_declares_the_workflow_argument_defaulting_to_the_selftest_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == SELFTEST_WORKFLOW
