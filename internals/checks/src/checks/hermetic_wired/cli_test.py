"""Colocated unit tests for the hermetic-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path is
asserted against the propagated exception's `.message`.
"""
from checks.hermetic_wired.cli import GUARD, REUSABLE_WORKFLOW, cli

WIRED = f"""
jobs:
  detect:
    steps:
      - id: scan_hermetic
        if: ${{{{ {GUARD} }}}}
        uses: ./.github/actions/detect
    outputs:
      cli_command: x
  unit-lint:
    steps:
      - uses: actions/download-artifact@v8
        with:
          name: hermetic-cli
      - run: ${{CLI_COMMAND:-npx -y "testing-conventions"}} unit lint
"""

UNWIRED = "jobs:\n  detect:\n    steps:\n      - uses: thekevinscott/testing-conventions/.github/actions/detect@v0\n"

CALLER_WIRED = """
jobs:
  build-cli:
    steps:
      - run: uv run --project internals/checks tc-checks build-hermetic-cli hermetic-cli-stage
  clean:
    needs: [build-cli]
    uses: ./.github/workflows/testing-conventions.yml
  packaging-clean:
    needs: [upload-clean-dist, build-cli]
    uses: ./.github/workflows/testing-conventions.yml
"""

CALLER_MISSING_NEEDS = CALLER_WIRED.replace("    needs: [build-cli]\n", "")


def _write(tmp_path, name, text):
    path = tmp_path / name
    path.write_text(text)
    return str(path)


def test_echoes_on_a_wired_workflow_with_wired_callers(tmp_path, capsys):
    workflow = _write(tmp_path, "wf.yml", WIRED)
    caller = _write(tmp_path, "caller.yml", CALLER_WIRED)
    cli.callback(workflow=workflow, callers=(caller,))
    assert "derived, caller-built, and fully wired" in capsys.readouterr().out


def test_raises_on_an_unwired_workflow(tmp_path):
    workflow = _write(tmp_path, "wf.yml", UNWIRED)
    try:
        cli.callback(workflow=workflow, callers=())
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "the derivation guard" in error.message
        assert "a local" in error.message
        assert "a `cli_command`" in error.message
        assert "the `${CLI_COMMAND:-` npx fallback" in error.message
        assert "a `hermetic-cli` artifact download" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_raises_with_only_the_missing_pieces_named(tmp_path):
    workflow = _write(tmp_path, "wf.yml", WIRED.replace("cli_command: x", ""))
    try:
        cli.callback(workflow=workflow, callers=())
    except Exception as error:  # noqa: BLE001
        assert "a `cli_command`" in error.message
        assert "the derivation guard" not in error.message
    else:
        raise AssertionError("a workflow missing only cli_command must raise")


def test_raises_on_a_flag_shaped_workflow_even_when_fully_wired(tmp_path):
    workflow = _write(tmp_path, "wf.yml", WIRED + "      - run: echo ${{ inputs.hermetic }}\n")
    try:
        cli.callback(workflow=workflow, callers=())
    except Exception as error:  # noqa: BLE001
        assert "inputs.hermetic" in error.message
        assert "never declared by an input" in error.message
    else:
        raise AssertionError("a workflow referencing inputs.hermetic must raise")


def test_raises_on_a_build_job_in_the_reusable_workflow(tmp_path):
    workflow = _write(tmp_path, "wf.yml", WIRED + "  build-cli:\n    runs-on: ubuntu-latest\n")
    try:
        cli.callback(workflow=workflow, callers=())
    except Exception as error:  # noqa: BLE001
        assert "declares a `build-cli` job" in error.message
        assert "skipped row" in error.message
    else:
        raise AssertionError("a build-cli job in the reusable workflow must raise")


def test_raises_when_a_caller_has_no_build_job(tmp_path):
    workflow = _write(tmp_path, "wf.yml", WIRED)
    caller = _write(tmp_path, "caller.yml", "jobs:\n  clean:\n    uses: ./.github/workflows/testing-conventions.yml\n")
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "has no `build-cli` job" in error.message
    else:
        raise AssertionError("a caller without a build-cli job must raise")


def test_raises_when_a_callers_uses_call_lacks_the_needs_edge(tmp_path):
    workflow = _write(tmp_path, "wf.yml", WIRED)
    caller = _write(tmp_path, "caller.yml", CALLER_MISSING_NEEDS)
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "2 `uses:` call(s)" in error.message
        assert "only 1 carry" in error.message
        assert "races" in error.message
    else:
        raise AssertionError("a uses: call without needs: [build-cli] must raise")


def test_raises_when_a_caller_has_more_needs_edges_than_uses_calls(tmp_path):
    # The uses/needs count check must catch a mismatch in either direction, not just
    # uses > needs: an extra needs: [... build-cli ...] line with no matching uses: call is
    # just as much a wiring drift as a missing one.
    workflow = _write(tmp_path, "wf.yml", WIRED)
    caller = _write(
        tmp_path,
        "caller.yml",
        CALLER_WIRED + "  extra:\n    needs: [build-cli]\n    run: echo hi\n",
    )
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "2 `uses:` call(s)" in error.message
        assert "only 3 carry" in error.message
        assert "races" in error.message
    else:
        raise AssertionError("more needs: [build-cli] edges than uses: calls must raise")


def test_declares_the_workflow_argument_and_variadic_callers():
    # Assert click's own registered metadata (the decorators) — `.callback` bypasses arg
    # parsing, so this is what pins them without a CliRunner collaborator.
    workflow, callers = cli.params
    assert workflow.name == "workflow"
    assert workflow.default == REUSABLE_WORKFLOW
    assert callers.name == "callers"
    assert callers.nargs == -1
