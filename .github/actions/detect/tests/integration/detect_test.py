"""Integration tests for `detect.compute_outputs`.

Per the standard, an integration test runs first-party code for real and mocks the externals.
The one external here is the filesystem, so each test patches the boundary functions
(`has_source` / `has_rust_crate` / `has_dist` / `has_attestation`) and drives the real
orchestration, asserting the emitted sets.
"""
import sys
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))  # .github/scripts
import detect  # noqa: E402


def _run(languages, *, python=False, typescript=False, rust_crate=False,
         dist=False, attestation=False):
    """Run compute_outputs with the filesystem fully mocked to the given facts."""
    present = {"python": python, "typescript": typescript}
    patches = [
        patch.object(detect, "has_source", lambda root, lang: present[lang]),
        patch.object(detect, "has_rust_crate", lambda root: rust_crate),
    ]
    # has_dist / has_attestation only exist once #186 lands; patch them when present so these
    # helpers don't themselves force the feature.
    if hasattr(detect, "has_dist"):
        patches.append(patch.object(detect, "has_dist", lambda root: dist))
    if hasattr(detect, "has_attestation"):
        patches.append(patch.object(detect, "has_attestation", lambda root: attestation))
    for p in patches:
        p.start()
    try:
        return detect.compute_outputs(languages, scan_root="/repo")
    finally:
        for p in patches:
            p.stop()


# --- baseline: current behavior (green) ---

def test_explicit_python_only():
    out = _run('["python"]', python=True)
    assert out["languages"] == '["python"]'
    assert out["integration_lint_languages"] == '["python"]'
    assert out["isolation_languages"] == '["python"]'
    assert out["coverage_languages"] == '["python"]'


def test_explicit_rust_routes_into_coverage_zero_config():
    # Rust coverage is zero-config now (#206): a crate enters the coverage matrix even
    # without a configured floor — the default `lines = 100` applies.
    out = _run('["rust"]', rust_crate=True)
    assert out["integration_lint_languages"] == '["rust"]'
    assert out["isolation_languages"] == '["rust"]'
    assert out["coverage_languages"] == '["rust"]'


def test_requested_but_absent_language_is_skipped():
    out = _run('["python","typescript"]', python=True, typescript=False)
    assert out["languages"] == '["python"]'


def test_restrictor_excludes_unnamed_language():
    out = _run('["python"]', python=True, rust_crate=True)
    assert out["integration_lint_languages"] == '["python"]'  # rust excluded by the restrictor


# --- #185 auto-detect (RED until implemented) ---

def test_empty_languages_auto_detects_rust():
    out = _run("", rust_crate=True)
    assert out["integration_lint_languages"] == '["rust"]'
    assert out["isolation_languages"] == '["rust"]'


def test_empty_languages_auto_detects_python():
    out = _run("", python=True)
    assert out["languages"] == '["python"]'


def test_restrictor_still_applies_when_set():
    out = _run('["python"]', python=True, rust_crate=True)
    assert "rust" not in out["integration_lint_languages"]


# --- #204 mutation_languages (RED until implemented) ---

def test_mutation_languages_mirrors_coverage_for_python():
    # The mutation matrix is the same set as coverage: present python/typescript plus
    # rust when a crate is here (all three arms are at parity now).
    out = _run('["python"]', python=True)
    assert out["mutation_languages"] == '["python"]'


def test_mutation_languages_includes_rust_crate():
    out = _run('["rust"]', rust_crate=True)
    assert out["mutation_languages"] == '["rust"]'


def test_mutation_languages_auto_detects_present_set():
    out = _run("", python=True, rust_crate=True)
    assert out["mutation_languages"] == '["python","rust"]'


# --- #186 packaging_dist / e2e_attestation (RED until implemented) ---

def test_packaging_dist_true_when_dist_present():
    out = _run("", dist=True)
    assert out["packaging_dist"] == "true"


def test_packaging_dist_false_when_absent():
    out = _run("", dist=False)
    assert out["packaging_dist"] == "false"


def test_e2e_attestation_true_when_present():
    out = _run("", attestation=True)
    assert out["e2e_attestation"] == "true"


def test_e2e_attestation_false_when_absent():
    out = _run("", attestation=False)
    assert out["e2e_attestation"] == "false"
