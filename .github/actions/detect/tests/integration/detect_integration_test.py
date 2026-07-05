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
        "package_root": Path("/repo"),
        "ts_package_manager": "pnpm",
        "python_env": "pip",
        "provision_rust": "false",
        "config": "testing-conventions.toml",
        "attestation_roots_seen": [],
    }

    def has_attestation(root):
        state["attestation_roots_seen"].append(root)
        return state["attestation"]

    with patch.object(detect, "has_source", lambda root, language: state[language]), \
            patch.object(detect, "has_rust_crate", lambda root: state["rust_crate"]), \
            patch.object(detect, "has_dist", lambda root: state["dist"]), \
            patch.object(detect, "has_attestation", has_attestation), \
            patch.object(detect, "derive_package_root", lambda scan_root, repo_root: state["package_root"]), \
            patch.object(detect, "ts_package_manager", lambda root: state["ts_package_manager"]), \
            patch.object(detect, "python_env", lambda root: state["python_env"]), \
            patch.object(detect, "provision_rust", lambda root: state["provision_rust"]), \
            patch.object(detect, "derive_config", lambda package_root_rel, config_input: state["config"]):
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
    assert out["languages"] == "[]"  # the co-change matrix stays python/typescript
    assert out["integration_lint_languages"] == '["rust"]'
    assert out["isolation_languages"] == '["rust"]'
    assert out["coverage_languages"] == '["rust"]'


def test_rust_crate_enters_the_colocated_test_matrix(fs):
    # #274: the CLI ships the rust presence arm (inline `#[cfg(test)]`, #40), so the
    # whole-tree colocated-test job fans out over rust too. Co-change stays on the
    # file-paired `languages` set — rust units are inline, so a sibling test can't
    # go stale.
    fs["rust_crate"] = True
    out = detect.compute_outputs('["rust"]', scan_root="/repo")
    assert out["colocated_test_languages"] == '["rust"]'
    assert out["languages"] == "[]"


def test_colocated_test_matrix_lists_rust_after_the_file_paired_languages(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["colocated_test_languages"] == '["python","rust"]'


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


def test_monorepo_outputs_wired_from_the_package_root(fs):
    fs["package_root"] = Path("/repo/packages/ts")
    fs["ts_package_manager"] = "npm"
    fs["python_env"] = "uv"
    fs["provision_rust"] = "true"
    out = detect.compute_outputs("", scan_root="/repo/packages/ts/src", repo_root="/repo")
    assert out["package_root"] == "packages/ts"
    assert out["ts_package_manager"] == "npm"
    assert out["python_env"] == "uv"
    assert out["provision_rust"] == "true"


def test_package_root_relative_is_dot_when_it_is_the_repo_root(fs):
    fs["package_root"] = Path("/repo")
    out = detect.compute_outputs("", scan_root="/repo/src", repo_root="/repo")
    assert out["package_root"] == "."


def test_config_output_is_wired_from_derive_config(fs):
    fs["config"] = "packages/ts/testing-conventions.toml"
    out = detect.compute_outputs("", scan_root="/repo/packages/ts/src", repo_root="/repo")
    assert out["config"] == "packages/ts/testing-conventions.toml"


def test_attestation_is_looked_up_at_the_package_root_not_the_repo_root(fs):
    # #281: `has_attestation` is called with `package_root`, not the checkout root — the
    # fixture's `has_attestation` records every root it receives, proving the wiring.
    fs["package_root"] = Path("/repo/packages/x")
    fs["attestation"] = True
    out = detect.compute_outputs("", scan_root="/repo/packages/x/src", repo_root="/repo")
    assert fs["attestation_roots_seen"] == [Path("/repo/packages/x")]
    assert out["e2e_attestation"] == "true"
