"""Integration tests for `compute_outputs`: the real orchestration, the filesystem mocked.

The one external is the filesystem, so the `fs` fixture patches the boundary functions to read
a mutable `state`; each test sets the facts it needs and asserts the emitted sets."""
from pathlib import Path
from unittest.mock import patch

import pytest

import compute_outputs


@pytest.fixture
def fs():
    """Patch the filesystem boundary to read a mutable `state`; yield it for per-test setup."""
    state = {
        "python": False,
        "typescript": False,
        "rust_crate": False,
        "dist": False,
        "dist_root": None,  # where "dist" is found; None means the derived package root
        "attestation": False,
        "package_root": Path("/repo"),
        "ts_package_manager": "pnpm",
        "ts_pnpm_version": ">=11",
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

    with patch.object(compute_outputs, "has_source", lambda root, language: state[language]), \
            patch.object(compute_outputs, "has_rust_crate", lambda root: state["rust_crate"]), \
            patch.object(
                compute_outputs, "has_dist",
                lambda root: state["dist"] and root == (state["dist_root"] or state["package_root"]),
            ), \
            patch.object(compute_outputs, "has_attestation", has_attestation), \
            patch.object(compute_outputs, "derive_package_root", lambda scan_root, repo_root: state["package_root"]), \
            patch.object(compute_outputs, "ts_package_manager", lambda root: state["ts_package_manager"]), \
            patch.object(compute_outputs, "ts_pnpm_version", lambda root: state["ts_pnpm_version"]), \
            patch.object(compute_outputs, "python_env", lambda root: state["python_env"]), \
            patch.object(compute_outputs, "provision_rust", lambda root: state["provision_rust"]), \
            patch.object(compute_outputs, "derive_config", lambda package_root_rel, config_input: state["config"]), \
            patch.object(compute_outputs, "primary_language", lambda package_root: state["primary"]), \
            patch.object(compute_outputs, "derive_packaging", lambda package_root, primary, repo_root: state["packaging_build"]), \
            patch.object(compute_outputs, "derive_build_command", lambda config, language: state["build_command"]), \
            patch.object(compute_outputs, "derive_e2e_extra_scope", lambda config: state["e2e_extra_scope"]), \
            patch.object(compute_outputs, "derive_e2e_exclude", lambda config: state["e2e_exclude"]):
        yield state


def test_explicit_python_only(fs):
    fs["python"] = True
    out = compute_outputs.compute_outputs('["python"]', scan_root="/repo")
    assert out["languages"] == '["python"]'
    assert out["coverage_languages"] == '["python"]'
    assert out["mutation_languages"] == '["python"]'


def test_rust_crate_routes_into_the_with_rust_sets(fs):
    fs["rust_crate"] = True
    out = compute_outputs.compute_outputs('["rust"]', scan_root="/repo")
    assert out["languages"] == "[]"  # the co-change matrix stays python/typescript
    assert out["integration_lint_languages"] == '["rust"]'
    assert out["isolation_languages"] == '["rust"]'
    assert out["coverage_languages"] == '["rust"]'


def test_rust_crate_enters_the_colocated_test_matrix(fs):
    fs["rust_crate"] = True
    out = compute_outputs.compute_outputs('["rust"]', scan_root="/repo")
    assert out["colocated_test_languages"] == '["rust"]'
    assert out["languages"] == "[]"


def test_colocated_test_matrix_lists_rust_after_the_file_paired_languages(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["colocated_test_languages"] == '["python","rust"]'


def test_static_languages_is_the_rust_inclusive_union(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["static_languages"] == '["python","rust"]'


def test_static_languages_empty_when_nothing_is_present(fs):
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["static_languages"] == "[]"


def test_one_function_languages_is_the_rust_inclusive_union(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["one_function_languages"] == '["python","rust"]'


def test_one_function_languages_empty_when_nothing_is_present(fs):
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["one_function_languages"] == "[]"


def test_one_function_languages_honours_the_restrictor(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = compute_outputs.compute_outputs('["python"]', scan_root="/repo")
    assert out["one_function_languages"] == '["python"]'


def test_restrictor_excludes_an_unnamed_language(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = compute_outputs.compute_outputs('["python"]', scan_root="/repo")
    assert out["integration_lint_languages"] == '["python"]'  # rust excluded by the restrictor


def test_empty_languages_auto_detects_the_present_set(fs):
    fs["python"] = True
    fs["rust_crate"] = True
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["languages"] == '["python"]'
    assert out["mutation_languages"] == '["python","rust"]'


def test_packaging_dist_and_attestation_flags(fs):
    fs["dist"] = True
    fs["attestation"] = True
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["packaging_dist"] == "true"
    assert out["e2e_attestation"] == "true"


def test_packaging_dist_and_attestation_absent(fs):
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["packaging_dist"] == "false"
    assert out["e2e_attestation"] == "false"


def test_packaging_dist_found_at_the_derived_package_root(fs):
    fs["package_root"] = Path("/repo/packages/x")
    fs["dist"] = True
    out = compute_outputs.compute_outputs("", scan_root="/repo/packages/x/src", repo_root="/repo")
    assert out["packaging_dist"] == "true"


def test_packaging_dist_at_the_repo_root_is_not_found_for_a_scoped_package(fs):
    fs["package_root"] = Path("/repo/packages/x")
    fs["dist"] = True
    fs["dist_root"] = Path("/repo")  # the dist sits at the checkout root, not the package root
    out = compute_outputs.compute_outputs("", scan_root="/repo/packages/x/src", repo_root="/repo")
    assert out["packaging_dist"] == "false"


def test_monorepo_outputs_wired_from_the_package_root(fs):
    fs["package_root"] = Path("/repo/packages/ts")
    fs["ts_package_manager"] = "npm"
    fs["python_env"] = "uv"
    fs["provision_rust"] = "true"
    out = compute_outputs.compute_outputs("", scan_root="/repo/packages/ts/src", repo_root="/repo")
    assert out["package_root"] == "packages/ts"
    assert out["ts_package_manager"] == "npm"
    assert out["python_env"] == "uv"
    assert out["provision_rust"] == "true"


def test_ts_pnpm_version_is_emitted_from_the_package_root(fs):
    fs["ts_pnpm_version"] = "10.33.0"
    out = compute_outputs.compute_outputs("", scan_root="/repo/src", repo_root="/repo")
    assert out["ts_pnpm_version"] == "10.33.0"


def test_package_root_relative_is_dot_when_it_is_the_repo_root(fs):
    fs["package_root"] = Path("/repo")
    out = compute_outputs.compute_outputs("", scan_root="/repo/src", repo_root="/repo")
    assert out["package_root"] == "."


def test_config_output_is_wired_from_derive_config(fs):
    fs["config"] = "packages/ts/testing-conventions.toml"
    out = compute_outputs.compute_outputs("", scan_root="/repo/packages/ts/src", repo_root="/repo")
    assert out["config"] == "packages/ts/testing-conventions.toml"


def test_build_command_output_wired_from_derive_build_command(fs):
    fs["build_command"] = "uv run maturin develop"
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["build_command"] == "uv run maturin develop"


def test_build_command_output_empty_by_default(fs):
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["build_command"] == ""


def test_packaging_build_output_wired_from_derive_packaging(fs):
    fs["primary"] = "python"
    fs["packaging_build"] = "uv build"
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["packaging_build"] == "uv build"
    assert out["packaging_language"] == "python"


def test_packaging_language_is_empty_when_no_build_was_derived(fs):
    fs["primary"] = "python"
    fs["packaging_build"] = ""
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["packaging_build"] == ""
    assert out["packaging_language"] == ""


def test_e2e_extra_scope_output_wired_from_derive(fs):
    fs["e2e_extra_scope"] = "--extra-scope packages/rust/src"
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["e2e_extra_scope"] == "--extra-scope packages/rust/src"


def test_e2e_exclude_output_wired_from_derive(fs):
    fs["e2e_exclude"] = "--exclude packages/rust/src/cli"
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["e2e_exclude"] == "--exclude packages/rust/src/cli"


def test_e2e_extra_scope_and_exclude_empty_by_default(fs):
    out = compute_outputs.compute_outputs("", scan_root="/repo")
    assert out["e2e_extra_scope"] == ""
    assert out["e2e_exclude"] == ""


def test_attestation_is_looked_up_at_the_package_root_not_the_repo_root(fs):
    fs["package_root"] = Path("/repo/packages/x")
    fs["attestation"] = True
    out = compute_outputs.compute_outputs("", scan_root="/repo/packages/x/src", repo_root="/repo")
    assert fs["attestation_roots_seen"] == [Path("/repo/packages/x")]
    assert out["e2e_attestation"] == "true"


def test_cli_command_is_the_hermetic_artifact_for_this_repos_caller(fs):
    outputs = compute_outputs.compute_outputs(
        "", "scan", caller_repository="thekevinscott/testing-conventions", version=""
    )
    assert outputs["cli_command"] == "./hermetic-cli/testing-conventions"
    assert (
        outputs["ts_mutation_adapter_args"]
        == "--ts-mutation-adapter ./hermetic-cli/dist/mutation/main.js"
    )


def test_cli_command_is_empty_for_any_other_caller(fs):
    outputs = compute_outputs.compute_outputs("", "scan", caller_repository="someone/else", version="")
    assert outputs["cli_command"] == ""
    assert outputs["ts_mutation_adapter_args"] == ""


def test_cli_command_is_empty_when_a_version_is_pinned(fs):
    outputs = compute_outputs.compute_outputs(
        "", "scan", caller_repository="thekevinscott/testing-conventions", version="0.3.0"
    )
    assert outputs["cli_command"] == ""


def test_cli_command_defaults_empty_when_the_caller_is_not_passed(fs):
    assert compute_outputs.compute_outputs("", "scan")["cli_command"] == ""
