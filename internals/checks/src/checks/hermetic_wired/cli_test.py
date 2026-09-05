"""Colocated unit tests for the hermetic-wired check."""
from checks.hermetic_wired.cli import GUARD, REUSABLE_WORKFLOW, cli

ENV_LINE = "          CLI_COMMAND: ${{ needs.detect.outputs.cli_command }}\n"
LAUNCHER = 'npm --prefix "$RUNNER_TEMP" exec --yes --'


def fallback_step(gate, wired=True):
    """A `steps:` list item running the `${CLI_COMMAND:-` fallback, with or without its env line."""
    env = "        env:\n" + ENV_LINE if wired else ""
    run = f'        run: ${{CLI_COMMAND:-{LAUNCHER} "testing-conventions"}} unit {gate}\n'
    return f"      - name: Check {gate}\n" + env + run


WIRED = (
    f"""
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
      - uses: ./.github/actions/download-hermetic-cli
"""
    + fallback_step("lint")
    + fallback_step("colocated-test")
)

UNWIRED_STEP = WIRED.replace(fallback_step("colocated-test"), fallback_step("colocated-test", wired=False))

UNWIRED = "jobs:\n  detect:\n    steps:\n      - uses: thekevinscott/testing-conventions/.github/actions/detect@v0\n"

CALLER_WIRED = """
jobs:
  build-cli:
    steps:
      - uses: ./.github/actions/build-hermetic-cli
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
        assert "the `${CLI_COMMAND:-` published-CLI fallback" in error.message
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


def test_raises_when_a_caller_inlines_the_build_steps_instead_of_the_composite_action(tmp_path):
    workflow = _write(tmp_path, "wf.yml", WIRED)
    caller = _write(
        tmp_path,
        "caller.yml",
        CALLER_WIRED.replace(
            "      - uses: ./.github/actions/build-hermetic-cli\n",
            "      - run: uv run --project internals/checks tc-checks build-hermetic-cli hermetic-cli-stage\n",
        ),
    )
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "doesn't call the shared" in error.message
        assert "build-hermetic-cli" in error.message
    else:
        raise AssertionError("a caller inlining the build steps instead of the composite action must raise")


def test_raises_when_a_callers_uses_call_lacks_the_needs_edge(tmp_path):
    workflow = _write(tmp_path, "wf.yml", WIRED)
    caller = _write(tmp_path, "caller.yml", CALLER_MISSING_NEEDS)
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "clean" in error.message
        assert "packaging-clean" not in error.message
        assert "races" in error.message
    else:
        raise AssertionError("a uses: call without needs: [build-cli] must raise")


def test_names_the_unwired_job_even_when_an_unrelated_job_has_the_edge(tmp_path):
    # An unrelated job carrying `needs: [build-cli]` with no `uses:` call of its own numerically
    # balances a genuinely unwired one, so a file-wide count passes while the race is real.
    workflow = _write(tmp_path, "wf.yml", WIRED)
    caller = _write(
        tmp_path,
        "caller.yml",
        CALLER_MISSING_NEEDS + "  extra:\n    needs: [build-cli]\n    run: echo hi\n",
    )
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "clean" in error.message
        assert "packaging-clean" not in error.message
        assert "races" in error.message
    else:
        raise AssertionError("an unrelated job's needs: edge must not mask a different job's missing edge")


def test_declares_the_workflow_argument_and_variadic_callers():
    workflow, callers = cli.params
    assert workflow.name == "workflow"
    assert workflow.default == REUSABLE_WORKFLOW
    assert callers.name == "callers"
    assert callers.nargs == -1


def test_raises_when_one_of_two_fallback_steps_lacks_its_own_cli_command_env(tmp_path):
    workflow = _write(tmp_path, "wf.yml", UNWIRED_STEP)
    caller = _write(tmp_path, "caller.yml", CALLER_WIRED)
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "Check colocated-test" in error.message
        assert "Check lint" not in error.message
        assert "the `${CLI_COMMAND:-` published-CLI fallback" in error.message
    else:
        raise AssertionError("a step running the fallback without its own CLI_COMMAND env must raise")


def test_the_file_wide_fallback_needle_survives_a_single_unwired_step(tmp_path):
    # Every step still carries the `${CLI_COMMAND:-` text, so a file-wide substring check cannot
    # tell one unwired step from none.
    assert "${CLI_COMMAND:-" in UNWIRED_STEP
    workflow = _write(tmp_path, "wf.yml", UNWIRED_STEP)
    caller = _write(tmp_path, "caller.yml", CALLER_WIRED)
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "Check colocated-test" in error.message
    else:
        raise AssertionError("an intact file-wide needle must not mask a single unwired step")


def test_raises_when_a_step_sets_cli_command_from_something_other_than_detect(tmp_path):
    hardcoded = WIRED.replace(ENV_LINE, "          CLI_COMMAND: ./hermetic-cli/testing-conventions\n", 1)
    workflow = _write(tmp_path, "wf.yml", hardcoded)
    caller = _write(tmp_path, "caller.yml", CALLER_WIRED)
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "Check lint" in error.message
    else:
        raise AssertionError("a CLI_COMMAND env not read from detect must raise")


def test_a_neighbouring_steps_env_does_not_satisfy_an_unwired_step(tmp_path):
    # The wired `lint` step sits directly above the unwired one, so a scan running forward from
    # the fallback line instead of bounding each step would find the neighbour's env line.
    trailing_job = "  packaging:\n    steps:\n      - run: echo hi\n"
    workflow = _write(tmp_path, "wf.yml", UNWIRED_STEP + trailing_job)
    caller = _write(tmp_path, "caller.yml", CALLER_WIRED)
    try:
        cli.callback(workflow=workflow, callers=(caller,))
    except Exception as error:  # noqa: BLE001
        assert "Check colocated-test" in error.message
    else:
        raise AssertionError("a neighbour's env line must not satisfy an unwired step")
