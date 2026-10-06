"""End-to-end tests for the published-detect-contract command: a real git repository, a real
`git show`, click's CliRunner, no mocks.

The published side of the contract only exists inside a git ref, so the fixture is a real
repository whose `v0` tag carries a chosen manifest. The #714 case is replayed against this
repo's own files: the real reusable workflow, paired with the real detect manifest minus the one
output that entered it after the tag last moved.
"""
import os
import subprocess
from pathlib import Path

from click.testing import CliRunner

from checks.published_detect_contract.cli import cli
from checks.published_detect_contract.published_manifest import DETECT_MANIFEST

REPO_ROOT = Path(__file__).resolve().parents[4]
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "testing-conventions.yml"
ACTION = REPO_ROOT / DETECT_MANIFEST


def publish(tmp_path, manifest):
    """A real git repository whose `v0` tag carries `manifest` as the detect action."""
    target = tmp_path / DETECT_MANIFEST
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(manifest)
    for argv in (
        ["git", "init", "--initial-branch", "main"],
        ["git", "add", "-A"],
        ["git", "-c", "user.email=t@t", "-c", "user.name=t", "commit", "-m", "published"],
        ["git", "tag", "v0"],
    ):
        subprocess.run(argv, cwd=tmp_path, check=True, capture_output=True)
    return tmp_path


def run_in(repo, *arguments):
    old = os.getcwd()
    os.chdir(repo)
    try:
        return CliRunner().invoke(cli, list(arguments))
    finally:
        os.chdir(old)


def without_output(manifest, name):
    """`manifest` with the `name:` output and its body dropped, as the tag carried it before."""
    lines = manifest.split("\n")
    start = lines.index(f"  {name}:")
    end = next(i for i in range(start + 1, len(lines)) if not lines[i].startswith("    "))
    return "\n".join(lines[:start] + lines[end:])


def test_the_real_workflow_against_a_manifest_missing_packaging_root_names_it(tmp_path):
    # The #714 regression: `packaging_root` entered detect after the tag last moved, so the
    # packaging step's `<PATH>` arrived empty on every version-pinned promotion run.
    manifest = without_output(ACTION.read_text(), "packaging_root")
    assert "packaging_root" not in manifest
    result = run_in(publish(tmp_path, manifest), str(WORKFLOW))
    assert result.exit_code == 1
    assert "::error::" in result.output
    assert "declares no packaging_root," in result.output


def test_the_real_workflow_against_the_real_manifest_passes(tmp_path):
    result = run_in(publish(tmp_path, ACTION.read_text()), str(WORKFLOW))
    assert result.exit_code == 0
    assert "declares every output the workflow consumes" in result.output


def test_dropping_any_single_consumed_output_from_the_published_manifest_names_that_output(tmp_path):
    manifest = ACTION.read_text()
    consumed = sorted(
        name
        for name in ("package_root", "cli_command", "e2e_attestation", "config")
        if f"  {name}:" in manifest
    )
    assert len(consumed) == 4
    for name in consumed:
        repo = publish(tmp_path / name, without_output(manifest, name))
        result = run_in(repo, str(WORKFLOW))
        assert result.exit_code == 1, f"{name} dropped but the check passed"
        assert f"declares no {name}," in result.output


def test_a_ref_the_repository_does_not_carry_fails_rather_than_passing_vacuously(tmp_path):
    result = run_in(publish(tmp_path, ACTION.read_text()), str(WORKFLOW), "--ref", "v9")
    assert result.exit_code == 1
    assert "exited" in result.output
