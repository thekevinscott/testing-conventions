"""Integration tests for `detect.compute_outputs`: the real orchestration, the filesystem mocked.

Per the standard, an integration test runs first-party code for real and mocks the externals. The
one external is the filesystem, so the `fs` fixture patches the boundary functions
(`has_source` / `has_rust_crate` / `has_dist` / `has_attestation`) to read a mutable `state`; each
test sets the facts it needs and asserts the emitted sets. The patching lives in the fixture, never
inline in a test body.
"""
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))  # .github/actions/detect
import detect  # noqa: E402


@pytest.fixture
def fs():
    """Patch the filesystem boundary to read a mutable `state`; yield it for per-test setup."""
    state = {
        "python": False,
        "typescript": False,
        "rust_crate": False,
        "dist": False,
        "attestation": False,
    }
    with patch.object(detect, "has_source", lambda root, language: state[language]), \
            patch.object(detect, "has_rust_crate", lambda root: state["rust_crate"]), \
            patch.object(detect, "has_dist", lambda root: state["dist"]), \
            patch.object(detect, "has_attestation", lambda root: state["attestation"]):
        yield state


def test_explicit_python_only(fs):
    fs["python"] = True
    out = detect.compute_outputs('["python"]', scan_root="/repo")
    assert out["languages"] == '["python"]'
    assert out["coverage_languages"] == '["python"]'
    assert out["mutation_languages"] == '["python"]'


def test_rust_crate_routes_into_the_with_rust_sets(fs):
    fs["rust_crate"] = True
    out = detect.compute_outputs('["rust"]', scan_root="/repo")
    assert out["languages"] == "[]"  # rust is not a colocated-test language
    assert out["integration_lint_languages"] == '["rust"]'
    assert out["isolation_languages"] == '["rust"]'
    assert out["coverage_languages"] == '["rust"]'


def test_restrictor_excludes_an_unnamed_language(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = detect.compute_outputs('["python"]', scan_root="/repo")
    assert out["integration_lint_languages"] == '["python"]'  # rust excluded by the restrictor


def test_empty_languages_auto_detects_the_present_set(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["languages"] == '["python"]'
    assert out["mutation_languages"] == '["python","rust"]'


def test_packaging_dist_and_attestation_flags(fs):
    fs["dist"] = True
    fs["attestation"] = True
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["packaging_dist"] == "true"
    assert out["e2e_attestation"] == "true"


def test_packaging_dist_and_attestation_absent(fs):
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["packaging_dist"] == "false"
    assert out["e2e_attestation"] == "false"
