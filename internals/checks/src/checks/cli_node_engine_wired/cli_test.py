"""Colocated unit tests for the cli-node-engine-wired command (isolation — no `CliRunner`).

The job finder, floor parse, and violation decisions are pinned beside their own modules; here the
command's own read-decide-report path is driven through `.callback()` over real files.
"""
import json

from checks.cli_node_engine_wired.cli import CLI_INVOCATION, NODE_PACKAGE_MANIFEST, REUSABLE_WORKFLOW, cli

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


def test_echoes_the_floor_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    manifest = tmp_path / "package.json"
    manifest.write_text(json.dumps({"engines": {"node": ">=24"}}))
    cli.callback(workflow=str(workflow), manifest=str(manifest))
    assert "all 1 CLI-invoking jobs provision node 24 or newer" in capsys.readouterr().out


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


def test_a_workflow_matching_no_cli_invocation_fails_rather_than_passing_vacuously(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED.replace(CLI_INVOCATION, "testing-conventions"))
    manifest = tmp_path / "package.json"
    manifest.write_text(json.dumps({"engines": {"node": ">=24"}}))
    try:
        cli.callback(workflow=str(workflow), manifest=str(manifest))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "inspected nothing and would pass at any node pin" in error.message
    else:
        raise AssertionError("a workflow matching no invocation must raise")


def test_declares_the_workflow_and_manifest_arguments_with_their_defaults():
    workflow, manifest = cli.params
    assert workflow.name == "workflow"
    assert workflow.default == REUSABLE_WORKFLOW
    assert manifest.name == "manifest"
    assert manifest.default == NODE_PACKAGE_MANIFEST
