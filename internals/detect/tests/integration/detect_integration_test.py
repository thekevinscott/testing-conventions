"""Integration tests for `detect.compute_outputs`: the real orchestration, the filesystem mocked.

Per the standard, an integration test runs first-party code for real and mocks the externals. The
one external is the filesystem, so the `fs` fixture patches the boundary functions
(`has_source` / `has_rust_crate` / `has_dist` / `has_attestation`) to read a mutable `state`; each
test sets the facts it needs and asserts the emitted sets. The patching lives in the fixture, never
inline in a test body.
"""
from pathlib import Path
from unittest.mock import patch

import pytest

import detect


@pytest.fixture
def fs():
    """Patch the filesystem boundary to read a mutable `state`; yield it for per-test setup."""
    state = {
        "python": False,
        "typescript": False,
        "rust_crate": False,
        "dist": False,
        "dist_root": None,  # where "dist" is found; None means the derived package root (#280)
        "attestation": False,
        "package_root": Path("/repo"),
        "ts_package_manager": "pnpm",
        "python_env": "pip",
        "provision_rust": "false",
        "config": "testing-conventions.toml",
        "build_command": "",
        "primary": "",
        "packaging_build": "",
        "e2e_extra_scope": "",
        "e2e_exclude": "",
        "attestation_roots_seen": [],
    }

    def has_attestation(root):
        state["attestation_roots_seen"].append(root)
        return state["attestation"]

    with patch.object(detect, "has_source", lambda root, language: state[language]), \
            patch.object(detect, "has_rust_crate", lambda root: state["rust_crate"]), \
            patch.object(
                detect, "has_dist",
                lambda root: state["dist"] and root == (state["dist_root"] or state["package_root"]),
            ), \
            patch.object(detect, "has_attestation", has_attestation), \
            patch.object(detect, "derive_package_root", lambda scan_root, repo_root: state["package_root"]), \
            patch.object(detect, "ts_package_manager", lambda root: state["ts_package_manager"]), \
            patch.object(detect, "python_env", lambda root: state["python_env"]), \
            patch.object(detect, "provision_rust", lambda root: state["provision_rust"]), \
            patch.object(detect, "derive_config", lambda package_root_rel, config_input: state["config"]), \
            patch.object(detect, "primary_language", lambda package_root: state["primary"]), \
            patch.object(detect, "derive_packaging", lambda package_root, primary, repo_root: state["packaging_build"]), \
            patch.object(detect, "derive_build_command", lambda config, language: state["build_command"]), \
            patch.object(detect, "derive_e2e_extra_scope", lambda config: state["e2e_extra_scope"]), \
            patch.object(detect, "derive_e2e_exclude", lambda config: state["e2e_exclude"]):
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


def test_static_languages_is_the_rust_inclusive_union(fs):
    # #410: the `static` job's matrix — the four static gates run as its steps — fans out over its
    # own `static_languages` set (the file-paired languages plus rust), named apart from the
    # colocated/isolation/integration sets so a future per-set divergence needs no workflow change.
    fs["python"] = True
    fs["rust_crate"] = True
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["static_languages"] == '["python","rust"]'


def test_static_languages_empty_when_nothing_is_present(fs):
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["static_languages"] == "[]"


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


# --- #280: packaging_dist is looked for at the derived package root, not the checkout root ---


def test_packaging_dist_found_at_the_derived_package_root(fs):
    fs["package_root"] = Path("/repo/packages/x")
    fs["dist"] = True
    out = detect.compute_outputs("", scan_root="/repo/packages/x/src", repo_root="/repo")
    assert out["packaging_dist"] == "true"


def test_packaging_dist_at_the_repo_root_is_not_found_for_a_scoped_package(fs):
    fs["package_root"] = Path("/repo/packages/x")
    fs["dist"] = True
    fs["dist_root"] = Path("/repo")  # the dist sits at the checkout root, not the package root
    out = detect.compute_outputs("", scan_root="/repo/packages/x/src", repo_root="/repo")
    assert out["packaging_dist"] == "false"


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


# --- #289: the [python].build_command escape hatch is emitted as an output ---


def test_build_command_output_wired_from_derive_build_command(fs):
    # compute_outputs emits a `build_command` output wired straight from `derive_build_command`
    # (which reads the package's own testing-conventions.toml, discovered at the package root like
    # `config`). The workflow's suite-executing jobs read it instead of the removed input.
    fs["build_command"] = "uv run maturin develop"
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["build_command"] == "uv run maturin develop"


def test_build_command_output_empty_by_default(fs):
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["build_command"] == ""


# --- #335: the derived packaging build + its provisioning language are emitted as outputs ---


def test_packaging_build_output_wired_from_derive_packaging(fs):
    # compute_outputs emits `packaging_build` straight from `derive_packaging`, and
    # `packaging_language` from the primary language when a build was derived — so the packaging
    # job provisions the toolchain and builds the distribution before scanning.
    fs["primary"] = "python"
    fs["packaging_build"] = "uv build"
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["packaging_build"] == "uv build"
    assert out["packaging_language"] == "python"


def test_packaging_language_is_empty_when_no_build_was_derived(fs):
    # A package whose manifest can't state a build (`derive_packaging` returns "") reports no
    # packaging language either, so the job provisions nothing and falls back to a committed dist.
    fs["primary"] = "python"
    fs["packaging_build"] = ""
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["packaging_build"] == ""
    assert out["packaging_language"] == ""


# --- #333: the [e2e] extra_scope / exclude roots are emitted as outputs, wired straight from
# their derive functions (which read the package's own discovered config, like build_command) ---


def test_e2e_extra_scope_output_wired_from_derive(fs):
    fs["e2e_extra_scope"] = "--extra-scope packages/rust/src"
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["e2e_extra_scope"] == "--extra-scope packages/rust/src"


def test_e2e_exclude_output_wired_from_derive(fs):
    fs["e2e_exclude"] = "--exclude packages/rust/src/cli"
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["e2e_exclude"] == "--exclude packages/rust/src/cli"


def test_e2e_extra_scope_and_exclude_empty_by_default(fs):
    out = detect.compute_outputs("", scan_root="/repo")
    assert out["e2e_extra_scope"] == ""
    assert out["e2e_exclude"] == ""


def test_attestation_is_looked_up_at_the_package_root_not_the_repo_root(fs):
    # #281: `has_attestation` is called with `package_root`, not the checkout root — the
    # fixture's `has_attestation` records every root it receives, proving the wiring.
    fs["package_root"] = Path("/repo/packages/x")
    fs["attestation"] = True
    out = detect.compute_outputs("", scan_root="/repo/packages/x/src", repo_root="/repo")
    assert fs["attestation_roots_seen"] == [Path("/repo/packages/x")]
    assert out["e2e_attestation"] == "true"


def test_cli_command_is_the_hermetic_artifact_for_this_repos_caller(fs):
    outputs = detect.compute_outputs(
        "", "scan", caller_repository="thekevinscott/testing-conventions", version=""
    )
    assert outputs["cli_command"] == "./hermetic-cli/testing-conventions"
    assert (
        outputs["ts_mutation_adapter_args"]
        == "--ts-mutation-adapter ./hermetic-cli/dist/mutation/main.js"
    )


def test_cli_command_is_empty_for_any_other_caller(fs):
    outputs = detect.compute_outputs("", "scan", caller_repository="someone/else", version="")
    assert outputs["cli_command"] == ""
    assert outputs["ts_mutation_adapter_args"] == ""


def test_cli_command_is_empty_when_a_version_is_pinned(fs):
    outputs = detect.compute_outputs(
        "", "scan", caller_repository="thekevinscott/testing-conventions", version="0.3.0"
    )
    assert outputs["cli_command"] == ""


def test_cli_command_defaults_empty_when_the_caller_is_not_passed(fs):
    assert detect.compute_outputs("", "scan")["cli_command"] == ""
