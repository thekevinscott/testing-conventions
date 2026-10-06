"""Colocated unit tests for the published-detect-contract command (isolation — the manifest read
and the parse are patched by string target, so no git runs)."""
from checks.published_detect_contract.cli import PUBLISHED_REF, REUSABLE_WORKFLOW, cli

WORKFLOW = """
jobs:
  packaging:
    env:
      PACKAGING_ROOT: ${{ needs.detect.outputs.packaging_root }}
  e2e-verify:
    env:
      ATTESTATION: ${{ needs.detect.outputs.e2e_attestation }}
"""


def _patch(monkeypatch, declared, refs=None, manifests=None):
    """Patch the manifest read to answer with a sentinel text, and the parse to answer `declared`."""
    monkeypatch.setattr(
        "checks.published_detect_contract.cli.published_manifest",
        lambda ref: (refs if refs is not None else []).append(ref) or f"manifest at {ref}",
    )
    monkeypatch.setattr(
        "checks.published_detect_contract.cli.declared_outputs",
        lambda manifest: (manifests if manifests is not None else []).append(manifest) or declared,
    )


def _write(tmp_path, text):
    path = tmp_path / "testing-conventions.yml"
    path.write_text(text)
    return str(path)


def test_echoes_when_the_published_manifest_declares_every_consumed_output(monkeypatch, tmp_path, capsys):
    refs, manifests = [], []
    _patch(monkeypatch, {"packaging_root", "e2e_attestation"}, refs, manifests)
    cli.callback(workflow=_write(tmp_path, WORKFLOW), ref="v0")
    assert refs == ["v0"]
    assert manifests == ["manifest at v0"]
    assert capsys.readouterr().out == (
        "the detect action published at v0 declares every output the workflow consumes\n"
    )


def test_raises_naming_the_consumed_outputs_the_published_manifest_omits(monkeypatch, tmp_path):
    _patch(monkeypatch, {"e2e_attestation"})
    try:
        cli.callback(workflow=_write(tmp_path, WORKFLOW), ref="v0")
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "published at v0 declares no packaging_root," in error.message
        assert "e2e_attestation" not in error.message
        assert error.message.endswith("the step runs as if the value were wired")
    else:
        raise AssertionError("a consumed output the published manifest omits must raise")


def test_the_absent_names_are_listed_in_sorted_order(monkeypatch, tmp_path):
    _patch(monkeypatch, set())
    try:
        cli.callback(workflow=_write(tmp_path, WORKFLOW), ref="v0")
    except Exception as error:  # noqa: BLE001
        assert "declares no e2e_attestation, packaging_root," in error.message
    else:
        raise AssertionError("two absent names must raise")


def test_an_output_the_manifest_declares_and_nobody_reads_is_not_a_failure(monkeypatch, tmp_path, capsys):
    # The assertion is a superset, not an equality: a published manifest ahead of the workflow is
    # the ordinary state of a tag that has moved past a consumption.
    _patch(monkeypatch, {"packaging_root", "e2e_attestation", "cargo_target_dir"})
    cli.callback(workflow=_write(tmp_path, WORKFLOW), ref="v0")
    assert "declares every output" in capsys.readouterr().out


def test_a_workflow_consuming_nothing_passes_without_reading_a_name(monkeypatch, tmp_path, capsys):
    _patch(monkeypatch, set())
    cli.callback(workflow=_write(tmp_path, "jobs:\n  detect:\n"), ref="v0")
    assert "declares every output" in capsys.readouterr().out


def test_the_ref_reaches_both_the_manifest_read_and_the_message(monkeypatch, tmp_path, capsys):
    refs = []
    _patch(monkeypatch, {"packaging_root", "e2e_attestation"}, refs)
    cli.callback(workflow=_write(tmp_path, WORKFLOW), ref="v0-candidate")
    assert refs == ["v0-candidate"]
    assert "published at v0-candidate declares every output" in capsys.readouterr().out


def test_declares_the_workflow_argument_and_the_published_ref_option():
    workflow, ref = cli.params
    assert workflow.name == "workflow"
    assert workflow.default == REUSABLE_WORKFLOW
    assert ref.name == "ref"
    assert ref.default == PUBLISHED_REF
    assert PUBLISHED_REF == "v0"
