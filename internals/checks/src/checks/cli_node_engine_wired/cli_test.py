"""Colocated unit tests for the cli-node-engine-wired check (isolation — no `CliRunner`)."""
import json

from checks.cli_node_engine_wired.cli import (
    NODE_PACKAGE_MANIFEST,
    REUSABLE_WORKFLOW,
    cli,
    engine_floor,
    violations,
)

WIRED = """\
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
  static:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/setup-node@v6
        with:
          node-version: 24
      - name: Check colocated test
        run: ${CLI_COMMAND:-npx -y "testing-conventions${VERSION:+@$VERSION}"} unit colocated-test
"""

GATED = WIRED.replace(
    "      - uses: actions/setup-node@v6",
    "      - if: matrix.language == 'typescript'\n        uses: actions/setup-node@v6",
)

TWO_NODES = WIRED.replace(
    "      - name: Check colocated test",
    "      - uses: actions/setup-node@v6\n        with:\n          node-version: 20\n"
    "      - name: Check colocated test",
)


def test_a_job_pinning_exactly_the_floor_is_no_violation():
    assert violations(WIRED, 24) == []


def test_a_job_with_no_setup_node_is_a_violation():
    text = WIRED.replace("      - uses: actions/setup-node@v6\n        with:\n          node-version: 24\n", "")
    assert violations(text, 24) == ["`static` invokes the CLI with no unconditional `setup-node` step"]


def test_a_setup_node_gated_by_an_if_does_not_count():
    assert violations(GATED, 24) == ["`static` invokes the CLI with no unconditional `setup-node` step"]


def test_a_job_pinning_above_the_floor_is_no_violation():
    assert violations(WIRED.replace("node-version: 24", "node-version: 26"), 24) == []


def test_a_job_pinning_below_the_floor_is_a_violation():
    text = WIRED.replace("node-version: 24", "node-version: 22")
    assert violations(text, 24) == ["`static` pins node 22, below the floor of 24"]


def test_the_highest_pinned_node_in_a_job_decides():
    assert violations(TWO_NODES, 24) == []
    assert violations(TWO_NODES, 26) == ["`static` pins node 24, below the floor of 26"]


def test_a_job_that_invokes_no_cli_needs_no_setup_node():
    assert violations("jobs:\n  detect:\n    runs-on: ubuntu-latest\n", 24) == []


def test_engine_floor_reads_a_bare_major():
    assert engine_floor(">=24") == 24


def test_engine_floor_reads_a_dotted_floor_and_surrounding_space():
    assert engine_floor(" >= 20.20.0 ") == 20


def test_engine_floor_rejects_a_requirement_it_cannot_read():
    try:
        engine_floor("^24")
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "`engines.node` is `^24`" in error.message
    else:
        raise AssertionError("an unreadable requirement must raise")


def test_echoes_the_floor_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    manifest = tmp_path / "package.json"
    manifest.write_text(json.dumps({"engines": {"node": ">=24"}}))
    cli.callback(workflow=str(workflow), manifest=str(manifest))
    assert "every CLI-invoking job provisions node 24 or newer" in capsys.readouterr().out


def test_raises_naming_every_problem_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(GATED)
    manifest = tmp_path / "package.json"
    manifest.write_text(json.dumps({"engines": {"node": ">=24"}}))
    try:
        cli.callback(workflow=str(workflow), manifest=str(manifest))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "resolves the bare name to an older release" in error.message
        assert "`static` invokes the CLI with no unconditional `setup-node` step" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_declares_the_workflow_and_manifest_arguments_with_their_defaults():
    workflow, manifest = cli.params
    assert workflow.name == "workflow"
    assert workflow.default == REUSABLE_WORKFLOW
    assert manifest.name == "manifest"
    assert manifest.default == NODE_PACKAGE_MANIFEST
