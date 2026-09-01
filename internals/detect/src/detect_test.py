"""Colocated unit tests for detect.

The filesystem is the one dependency, and it is exercised for real: every helper that reads a
tree is driven against a package built under `tmp_path`, so the assertions sit beside the code
they measure rather than behind a mock of it. `compute_outputs` and `main` run the same way,
against a real tree and a real GITHUB_OUTPUT file.
"""
import os
import runpy
from pathlib import Path
from unittest.mock import patch

import pytest

import detect


@pytest.fixture
def in_dir():
    """Enter a directory for the duration of a test, restoring the original on the way out.

    detect resolves `scan_root`, `repo_root`, and the derived config path relative to the
    process working directory, so the orchestration can only be driven from inside a tree.
    """
    original = Path.cwd()
    yield os.chdir
    os.chdir(original)


def write(path: Path, text: str = "") -> Path:
    """Create `path` (and its parents) holding `text`."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    return path


def test_eligible_empty_restrictor_allows_any_language():
    assert detect.eligible("", "python") is True


def test_eligible_empty_array_allows_any_language():
    assert detect.eligible("[]", "rust") is True


def test_eligible_named_language_is_in_scope():
    assert detect.eligible('["python"]', "python") is True


def test_eligible_unnamed_language_is_excluded():
    assert detect.eligible('["python"]', "rust") is False


def test_to_json_is_compact():
    assert detect._to_json(["python", "rust"]) == '["python","rust"]'


def test_to_json_empty_is_brackets():
    assert detect._to_json([]) == "[]"


def test_package_manager_from_field_extracts_the_name():
    assert detect._package_manager_from_field("pnpm@8.6.0") == "pnpm"


def test_package_manager_from_field_empty_is_none():
    assert detect._package_manager_from_field("") is None


def test_pnpm_version_pin_echoes_a_packagemanager_pin():
    assert detect._pnpm_version_pin("pnpm@10.33.0") == "10.33.0"


def test_pnpm_version_pin_echoes_a_pin_that_already_satisfies_the_floor():
    assert detect._pnpm_version_pin("pnpm@11.11.0") == "11.11.0"


def test_pnpm_version_pin_keeps_build_metadata_in_an_echoed_pin():
    pin = "11.11.0+sha512.abc123"
    assert detect._pnpm_version_pin(f"pnpm@{pin}") == pin


def test_pnpm_version_pin_is_never_empty_for_a_pnpm_pin():
    assert detect._pnpm_version_pin("pnpm@10.33.0") != ""


def test_pnpm_version_pin_falls_back_to_the_floor_with_no_field():
    assert detect._pnpm_version_pin("") == detect.PNPM_FLOOR


@pytest.mark.parametrize("declared", ["npm@10.0.0", "yarn@4.0.0"])
def test_pnpm_version_pin_falls_back_to_the_floor_for_another_manager(declared):
    assert detect._pnpm_version_pin(declared) == detect.PNPM_FLOOR


def test_build_command_language_prefers_primary():
    assert detect.build_command_language("python", ["python", "typescript"]) == "python"


def test_build_command_language_falls_back_to_the_sole_present_language():
    assert detect.build_command_language("", ["python"]) == "python"


def test_build_command_language_empty_when_no_manifest_and_none_present():
    assert detect.build_command_language("", []) == ""


def test_build_command_language_empty_when_no_manifest_and_ambiguous():
    assert detect.build_command_language("", ["python", "typescript"]) == ""


def test_derive_cargo_target_dir_standalone_uses_the_package_root():
    assert detect.derive_cargo_target_dir(Path("packages/rust"), None) == "packages/rust/target"


def test_derive_cargo_target_dir_workspace_member_uses_the_workspace_root():
    assert detect.derive_cargo_target_dir(Path("packages/rust"), Path(".")) == "./target"


def test_hermetic_for_this_repos_own_caller_with_no_version():
    # Built at runtime, so it is equal to the constant without being the same interned object:
    # comparing the caller by identity would leave every real run on the published path.
    caller = "/".join(["thekevinscott", "testing-conventions"])
    assert detect.hermetic(caller, "") is True


@pytest.mark.parametrize("caller", ["someone/else", "zzz/after-this-repo"])
def test_not_hermetic_for_any_other_caller(caller):
    assert detect.hermetic(caller, "") is False


def _parse_github_output(text):
    """Parse a GITHUB_OUTPUT file the way the Actions runner does: `name=value` lines
    plus the heredoc `name<<DELIM` / body / `DELIM` form for multi-line values."""
    result = {}
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if "<<" in line and "=" not in line.split("<<", 1)[0]:
            name, delim = line.split("<<", 1)
            i += 1
            body = []
            while i < len(lines) and lines[i] != delim:
                body.append(lines[i])
                i += 1
            result[name] = "\n".join(body)
            i += 1  # skip the closing delimiter
        elif "=" in line:
            name, value = line.split("=", 1)
            result[name] = value
            i += 1
        else:
            i += 1  # blank/trailing line
    return result


def test_render_github_output_writes_single_line_values_as_name_equals_value():
    rendered = detect.render_github_output({"languages": '["python"]', "package_root": "."})
    assert _parse_github_output(rendered) == {"languages": '["python"]', "package_root": "."}


def test_render_github_output_round_trips_a_multiline_value():
    outputs = {
        "build_command": "cp a.tmpl a.py\ncp b.tmpl b.py",
        "languages": '["python"]',
    }
    rendered = detect.render_github_output(outputs)
    assert _parse_github_output(rendered) == outputs


def test_an_explicit_version_wins_over_hermetic():
    assert detect.hermetic("thekevinscott/testing-conventions", "0.3.0") is False


def test_not_hermetic_when_the_caller_is_unknown():
    assert detect.hermetic("", "") is False


@pytest.mark.parametrize(
    ("language", "filename"),
    [
        ("python", "mod.py"),
        ("typescript", "mod.ts"),
        ("typescript", "view.tsx"),
        ("typescript", "mod.mts"),
        ("typescript", "mod.cts"),
    ],
)
def test_has_source_finds_a_language_file_at_any_depth(tmp_path, language, filename):
    write(tmp_path / "a" / "b" / filename)
    assert detect.has_source(tmp_path, language) is True


def test_has_source_ignores_another_languages_files(tmp_path):
    write(tmp_path / "mod.ts")
    assert detect.has_source(tmp_path, "python") is False


def test_has_source_is_false_for_an_empty_tree(tmp_path):
    assert detect.has_source(tmp_path, "typescript") is False


def test_has_rust_crate_needs_a_manifest_and_real_source(tmp_path):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    write(tmp_path / "src" / "lib.rs", "pub fn f() {}\n")
    assert detect.has_rust_crate(tmp_path) is True


def test_has_rust_crate_is_false_for_a_manifest_with_no_source(tmp_path):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert detect.has_rust_crate(tmp_path) is False


def test_has_rust_crate_is_false_for_source_with_no_manifest(tmp_path):
    write(tmp_path / "src" / "lib.rs", "pub fn f() {}\n")
    assert detect.has_rust_crate(tmp_path) is False


@pytest.mark.parametrize(
    "artifact",
    ["pkg-1.0-py3-none-any.whl", "pkg-1.0.tar.gz", "pkg-1.0.tgz", "pkg-1.0.crate"],
)
def test_has_dist_recognizes_each_built_distribution(tmp_path, artifact):
    write(tmp_path / "dist" / artifact)
    assert detect.has_dist(tmp_path) is True


def test_has_dist_ignores_an_unrecognized_file_in_dist(tmp_path):
    write(tmp_path / "dist" / "README.md")
    assert detect.has_dist(tmp_path) is False


def test_has_dist_is_false_without_a_dist_directory(tmp_path):
    write(tmp_path / "pkg-1.0.tar.gz")
    assert detect.has_dist(tmp_path) is False


def test_has_dist_is_false_when_dist_is_a_file(tmp_path):
    write(tmp_path / "dist")
    assert detect.has_dist(tmp_path) is False


def test_has_attestation_accepts_the_legacy_single_receipt(tmp_path):
    write(tmp_path / "e2e-attestation.json", "{}")
    assert detect.has_attestation(tmp_path) is True


def test_has_attestation_accepts_a_branch_keyed_receipt(tmp_path):
    write(tmp_path / "e2e-attestations" / "main.json", "{}")
    assert detect.has_attestation(tmp_path) is True


@pytest.mark.parametrize("entry", ["README.md", "main.cfg"])
def test_has_attestation_ignores_a_non_json_entry(tmp_path, entry):
    write(tmp_path / "e2e-attestations" / entry)
    assert detect.has_attestation(tmp_path) is False


def test_has_attestation_ignores_a_directory_named_like_a_receipt(tmp_path):
    (tmp_path / "e2e-attestations" / "main.json").mkdir(parents=True)
    assert detect.has_attestation(tmp_path) is False


def test_has_attestation_is_false_with_no_receipts(tmp_path):
    assert detect.has_attestation(tmp_path) is False


@pytest.mark.parametrize("manifest", ["package.json", "pyproject.toml", "Cargo.toml"])
def test_has_manifest_recognizes_each_manifest(tmp_path, manifest):
    write(tmp_path / manifest)
    assert detect.has_manifest(tmp_path) is True


def test_has_manifest_ignores_a_manifest_in_a_subdirectory(tmp_path):
    write(tmp_path / "sub" / "pyproject.toml")
    assert detect.has_manifest(tmp_path) is False


def test_read_package_json_parses_the_manifest(tmp_path):
    write(tmp_path / "package.json", '{"packageManager": "pnpm@9.1.0"}')
    assert detect.read_package_json(tmp_path) == {"packageManager": "pnpm@9.1.0"}


def test_read_package_json_is_empty_when_absent(tmp_path):
    assert detect.read_package_json(tmp_path) == {}


def test_read_package_json_is_empty_when_malformed(tmp_path):
    write(tmp_path / "package.json", "{not json")
    assert detect.read_package_json(tmp_path) == {}


def test_read_pyproject_parses_the_manifest(tmp_path):
    write(tmp_path / "pyproject.toml", '[project]\nname = "p"\n')
    assert detect.read_pyproject(tmp_path) == {"project": {"name": "p"}}


def test_read_pyproject_is_empty_when_absent(tmp_path):
    assert detect.read_pyproject(tmp_path) == {}


def test_read_pyproject_is_empty_when_malformed(tmp_path):
    write(tmp_path / "pyproject.toml", "[project\n")
    assert detect.read_pyproject(tmp_path) == {}


def test_has_lockfile_finds_the_named_file(tmp_path):
    write(tmp_path / "pnpm-lock.yaml")
    assert detect.has_lockfile(tmp_path, "pnpm-lock.yaml") is True


def test_has_lockfile_is_false_for_another_name(tmp_path):
    write(tmp_path / "pnpm-lock.yaml")
    assert detect.has_lockfile(tmp_path, "package-lock.json") is False


def test_has_lockfile_is_false_for_a_directory_of_that_name(tmp_path):
    (tmp_path / "pnpm-lock.yaml").mkdir()
    assert detect.has_lockfile(tmp_path, "pnpm-lock.yaml") is False


def test_derive_package_root_finds_the_nearest_manifest_above_the_scan(tmp_path):
    write(tmp_path / "packages" / "app" / "pyproject.toml")
    scan = tmp_path / "packages" / "app" / "src"
    scan.mkdir()
    assert detect.derive_package_root(scan, tmp_path) == (tmp_path / "packages" / "app").resolve()


def test_derive_package_root_prefers_the_scan_root_over_an_ancestor(tmp_path):
    write(tmp_path / "pyproject.toml")
    write(tmp_path / "packages" / "app" / "package.json")
    scan = tmp_path / "packages" / "app"
    assert detect.derive_package_root(scan, tmp_path) == scan.resolve()


def test_derive_package_root_falls_back_to_the_repo_root_with_no_manifest(tmp_path):
    scan = tmp_path / "packages" / "app" / "src"
    scan.mkdir(parents=True)
    assert detect.derive_package_root(scan, tmp_path) == tmp_path.resolve()


def test_derive_package_root_finds_a_manifest_above_a_scan_outside_the_repo(tmp_path):
    write(tmp_path / "aside" / "pyproject.toml")
    scan = tmp_path / "aside" / "src"
    scan.mkdir()
    (tmp_path / "repo").mkdir()
    assert detect.derive_package_root(scan, tmp_path / "repo") == (tmp_path / "aside").resolve()


def test_derive_package_root_stops_at_the_repo_root(tmp_path):
    write(tmp_path / "outer" / "pyproject.toml")
    repo = tmp_path / "outer" / "repo"
    scan = repo / "src"
    scan.mkdir(parents=True)
    assert detect.derive_package_root(scan, repo) == repo.resolve()


def test_ts_package_manager_prefers_the_declared_field(tmp_path):
    write(tmp_path / "package.json", '{"packageManager": "yarn@4.0.0"}')
    write(tmp_path / "pnpm-lock.yaml")
    assert detect.ts_package_manager(tmp_path) == "yarn"


def test_ts_package_manager_reads_a_pnpm_lockfile(tmp_path):
    write(tmp_path / "pnpm-lock.yaml")
    assert detect.ts_package_manager(tmp_path) == "pnpm"


def test_ts_package_manager_reads_an_npm_lockfile(tmp_path):
    write(tmp_path / "package-lock.json")
    assert detect.ts_package_manager(tmp_path) == "npm"


def test_ts_package_manager_defaults_to_pnpm(tmp_path):
    assert detect.ts_package_manager(tmp_path) == "pnpm"


def test_ts_pnpm_version_echoes_the_manifest_pin(tmp_path):
    write(tmp_path / "package.json", '{"packageManager": "pnpm@10.4.1"}')
    assert detect.ts_pnpm_version(tmp_path) == "10.4.1"


def test_ts_pnpm_version_falls_back_to_the_floor_without_a_pin(tmp_path):
    assert detect.ts_pnpm_version(tmp_path) == detect.PNPM_FLOOR


def test_python_env_is_uv_for_an_installable_project(tmp_path):
    write(tmp_path / "pyproject.toml", '[project]\nname = "p"\n')
    assert detect.python_env(tmp_path) == "uv"


def test_python_env_is_pip_for_a_tool_only_pyproject(tmp_path):
    write(tmp_path / "pyproject.toml", "[tool.ruff]\nline-length = 100\n")
    assert detect.python_env(tmp_path) == "pip"


def test_python_env_is_pip_without_a_pyproject(tmp_path):
    assert detect.python_env(tmp_path) == "pip"


def test_provision_rust_for_a_cargo_manifest(tmp_path):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert detect.provision_rust(tmp_path) == "true"


def test_provision_rust_for_a_maturin_backend(tmp_path):
    write(tmp_path / "pyproject.toml", '[build-system]\nbuild-backend = "maturin"\n')
    assert detect.provision_rust(tmp_path) == "true"


def test_provision_rust_ignores_another_build_backend(tmp_path):
    write(tmp_path / "pyproject.toml", '[build-system]\nbuild-backend = "hatchling.build"\n')
    assert detect.provision_rust(tmp_path) == "false"


def test_provision_rust_for_a_napi_key(tmp_path):
    write(tmp_path / "package.json", '{"napi": {"name": "core"}}')
    assert detect.provision_rust(tmp_path) == "true"


def test_provision_rust_for_a_napi_cli_devdependency(tmp_path):
    write(tmp_path / "package.json", '{"devDependencies": {"@napi-rs/cli": "^2.0.0"}}')
    assert detect.provision_rust(tmp_path) == "true"


def test_provision_rust_is_false_for_a_plain_node_package(tmp_path):
    write(tmp_path / "package.json", '{"devDependencies": {"vitest": "^3.0.0"}}')
    assert detect.provision_rust(tmp_path) == "false"


@pytest.mark.parametrize("override", ["./custom.toml", "vendor/custom.toml"])
def test_derive_config_returns_an_explicit_override_verbatim(tmp_path, in_dir, override):
    in_dir(tmp_path)
    write(tmp_path / "packages" / "app" / "testing-conventions.toml")
    assert detect.derive_config(Path("packages/app"), override) == override


def test_derive_config_discovers_the_file_at_the_package_root(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "packages" / "app" / "testing-conventions.toml")
    derived = detect.derive_config(Path("packages/app"), "testing-conventions.toml")
    assert derived == str(Path("packages/app/testing-conventions.toml"))


def test_derive_config_falls_back_to_the_default_when_the_package_has_none(tmp_path, in_dir):
    in_dir(tmp_path)
    (tmp_path / "packages" / "app").mkdir(parents=True)
    assert detect.derive_config(Path("packages/app"), "testing-conventions.toml") == "testing-conventions.toml"


def test_derive_build_command_reads_the_language_table(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = "make protos"\n')
    assert detect.derive_build_command("tc.toml", "python") == "make protos"


def test_derive_build_command_is_empty_for_another_language(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = "make protos"\n')
    assert detect.derive_build_command("tc.toml", "typescript") == ""


def test_derive_build_command_is_empty_without_a_language(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = "make protos"\n')
    assert detect.derive_build_command("tc.toml", "") == ""


def test_derive_build_command_is_empty_when_the_config_is_absent(tmp_path, in_dir):
    in_dir(tmp_path)
    assert detect.derive_build_command("tc.toml", "python") == ""


def test_derive_build_command_is_empty_when_the_config_is_malformed(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", "[python\n")
    assert detect.derive_build_command("tc.toml", "python") == ""


def test_derive_build_command_ignores_a_non_string_declaration(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = ["make", "protos"]\n')
    assert detect.derive_build_command("tc.toml", "python") == ""


def test_derive_build_command_keeps_a_multiline_declaration(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[python]\nbuild_command = """\ncp a.tmpl a.py\ncp b.tmpl b.py\n"""\n')
    assert detect.derive_build_command("tc.toml", "python") == "cp a.tmpl a.py\ncp b.tmpl b.py\n"


@pytest.mark.parametrize(
    ("manifest", "expected"),
    [("pyproject.toml", "python"), ("package.json", "typescript"), ("Cargo.toml", "rust")],
)
def test_primary_language_reads_each_manifest(tmp_path, manifest, expected):
    write(tmp_path / manifest)
    assert detect.primary_language(tmp_path) == expected


def test_primary_language_prefers_python_for_a_pyo3_binding(tmp_path):
    write(tmp_path / "pyproject.toml")
    write(tmp_path / "Cargo.toml")
    assert detect.primary_language(tmp_path) == "python"


def test_primary_language_prefers_typescript_for_a_napi_binding(tmp_path):
    write(tmp_path / "package.json")
    write(tmp_path / "Cargo.toml")
    assert detect.primary_language(tmp_path) == "typescript"


def test_primary_language_is_empty_without_a_manifest(tmp_path):
    assert detect.primary_language(tmp_path) == ""


def test_read_cargo_parses_the_manifest(tmp_path):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert detect.read_cargo(tmp_path) == {"package": {"name": "c"}}


def test_read_cargo_is_empty_when_absent(tmp_path):
    assert detect.read_cargo(tmp_path) == {}


def test_read_cargo_is_empty_when_malformed(tmp_path):
    write(tmp_path / "Cargo.toml", "[package\n")
    assert detect.read_cargo(tmp_path) == {}


def test_cargo_workspace_root_finds_the_owning_ancestor(tmp_path):
    write(tmp_path / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert detect.cargo_workspace_root(member, tmp_path) == tmp_path.resolve()


def test_cargo_workspace_root_is_none_without_a_workspace_ancestor(tmp_path):
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert detect.cargo_workspace_root(member, tmp_path) is None


def test_cargo_workspace_root_ignores_the_package_roots_own_workspace_table(tmp_path):
    root = tmp_path / "crate"
    write(root / "Cargo.toml", '[package]\nname = "c"\n\n[workspace]\n')
    assert detect.cargo_workspace_root(root, tmp_path) is None


def test_cargo_workspace_root_is_none_when_the_package_is_the_repo_root(tmp_path):
    write(tmp_path / "Cargo.toml", '[workspace]\n')
    assert detect.cargo_workspace_root(tmp_path, tmp_path) is None


def test_cargo_workspace_root_prefers_the_nearest_workspace(tmp_path):
    write(tmp_path / "Cargo.toml", '[workspace]\n')
    inner = tmp_path / "sub"
    write(inner / "Cargo.toml", '[workspace]\n')
    member = inner / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert detect.cargo_workspace_root(member, tmp_path) == inner.resolve()


def test_cargo_workspace_root_ignores_a_workspace_above_the_repo_root(tmp_path):
    write(tmp_path / "Cargo.toml", "[workspace]\n")
    member = tmp_path / "repo" / "crate"
    write(member / "Cargo.toml", '[package]\nname = "c"\n')
    assert detect.cargo_workspace_root(member, tmp_path / "repo") is None


def test_cargo_workspace_root_finds_a_workspace_above_a_package_outside_the_repo(tmp_path):
    write(tmp_path / "Cargo.toml", "[workspace]\n")
    member = tmp_path / "aside" / "crate"
    write(member / "Cargo.toml", '[package]\nname = "c"\n')
    (tmp_path / "repo").mkdir()
    assert detect.cargo_workspace_root(member, tmp_path / "repo") == tmp_path.resolve()


def test_is_workspace_member_for_a_crate_under_a_workspace(tmp_path):
    write(tmp_path / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert detect.is_workspace_member(member, tmp_path) is True


def test_is_workspace_member_is_false_for_a_standalone_crate(tmp_path):
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert detect.is_workspace_member(member, tmp_path) is False


def test_is_workspace_member_is_false_for_a_workspace_root_package(tmp_path):
    root = tmp_path / "crate"
    write(root / "Cargo.toml", '[package]\nname = "c"\n\n[workspace]\n')
    assert detect.is_workspace_member(root, tmp_path) is False


def test_is_workspace_member_is_false_when_the_package_is_the_repo_root(tmp_path):
    write(tmp_path / "Cargo.toml", '[workspace]\n')
    assert detect.is_workspace_member(tmp_path, tmp_path) is False


def test_derive_packaging_builds_a_python_project_with_uv(tmp_path):
    write(tmp_path / "pyproject.toml", '[project]\nname = "p"\n')
    assert detect.derive_packaging(tmp_path, "python", tmp_path) == "uv build"


def test_derive_packaging_is_empty_for_a_tool_only_pyproject(tmp_path):
    write(tmp_path / "pyproject.toml", "[tool.ruff]\n")
    assert detect.derive_packaging(tmp_path, "python", tmp_path) == ""


def test_derive_packaging_packs_a_typescript_package_with_its_manager(tmp_path):
    write(tmp_path / "package.json", '{"packageManager": "npm@10.0.0"}')
    assert detect.derive_packaging(tmp_path, "typescript", tmp_path) == "npm pack --pack-destination dist"


def test_derive_packaging_packages_a_standalone_crate(tmp_path):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert detect.derive_packaging(tmp_path, "rust", tmp_path) == "cargo package"


def test_derive_packaging_redirects_a_workspace_members_target_dir(tmp_path):
    write(tmp_path / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert detect.derive_packaging(member, "rust", tmp_path) == "cargo package --target-dir target"


def test_derive_packaging_is_empty_for_a_workspace_only_manifest(tmp_path):
    write(tmp_path / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    assert detect.derive_packaging(tmp_path, "rust", tmp_path) == ""


def test_derive_packaging_is_empty_for_a_language_the_table_does_not_name(tmp_path):
    write(tmp_path / "pyproject.toml", '[project]\nname = "p"\n')
    assert detect.derive_packaging(tmp_path, "", tmp_path) == ""


def test_derive_e2e_extra_scope_renders_repeated_flags(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nextra_scope = ["core", "shared/proto"]\n')
    assert detect.derive_e2e_extra_scope("tc.toml") == "--extra-scope core --extra-scope shared/proto"


def test_derive_e2e_exclude_renders_repeated_flags(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nexclude = ["core/cli"]\n')
    assert detect.derive_e2e_exclude("tc.toml") == "--exclude core/cli"


def test_e2e_scope_flags_is_empty_when_the_config_is_absent(tmp_path, in_dir):
    in_dir(tmp_path)
    assert detect.derive_e2e_extra_scope("tc.toml") == ""


def test_e2e_scope_flags_is_empty_when_the_config_is_malformed(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", "[e2e\n")
    assert detect.derive_e2e_extra_scope("tc.toml") == ""


def test_e2e_scope_flags_is_empty_when_the_key_is_absent(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nexclude = ["core/cli"]\n')
    assert detect.derive_e2e_extra_scope("tc.toml") == ""


def test_e2e_scope_flags_is_empty_for_a_non_list_declaration(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nextra_scope = "core"\n')
    assert detect.derive_e2e_extra_scope("tc.toml") == ""


def test_e2e_scope_flags_drops_non_string_and_empty_entries(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "tc.toml", '[e2e]\nextra_scope = ["core", "", 7]\n')
    assert detect.derive_e2e_extra_scope("tc.toml") == "--extra-scope core"


def test_free_of_keeps_an_uncontested_base():
    assert detect._free_of("delim", ["one", "two"]) == "delim"


def test_free_of_bumps_past_a_colliding_line():
    assert detect._free_of("delim", ["delim"]) == "delim_1"


def test_free_of_bumps_until_the_suffix_is_free():
    assert detect._free_of("delim", ["delim", "delim_1", "delim_2"]) == "delim_3"


def test_output_delimiter_appears_on_no_line_of_the_value():
    value = "cp a.tmpl a.py\ncp b.tmpl b.py"
    assert detect._output_delimiter(value) not in value.split("\n")


def test_output_delimiter_is_deterministic_for_a_value():
    assert detect._output_delimiter("same") == detect._output_delimiter("same")


def test_output_delimiter_differs_between_values():
    assert detect._output_delimiter("one") != detect._output_delimiter("two")


def build_ts_package(root: Path) -> None:
    """A TypeScript package with every manifest-derived output set to a distinctive value."""
    write(root / "package.json", '{"packageManager": "pnpm@10.4.1", "napi": {"name": "core"}}')
    write(
        root / "testing-conventions.toml",
        '[typescript]\nbuild_command = "pnpm build"\n\n[e2e]\nextra_scope = ["core"]\nexclude = ["core/cli"]\n',
    )
    write(root / "dist" / "pkg-1.0.tgz")
    write(root / "e2e-attestation.json", "{}")
    write(root / "src" / "mod.ts")
    write(root / "src" / "mod.py")


def test_compute_outputs_derives_every_output_from_the_package(tmp_path, in_dir):
    in_dir(tmp_path)
    build_ts_package(tmp_path / "packages" / "app")
    assert detect.compute_outputs(
        "",
        "packages/app/src",
        caller_repository="thekevinscott/testing-conventions",
    ) == {
        "languages": '["python","typescript"]',
        "colocated_test_languages": '["python","typescript"]',
        "integration_lint_languages": '["python","typescript"]',
        "isolation_languages": '["python","typescript"]',
        "static_languages": '["python","typescript"]',
        "one_function_languages": '["python","typescript"]',
        "coverage_languages": '["python","typescript"]',
        "mutation_languages": '["python","typescript"]',
        "packaging_dist": "true",
        "e2e_attestation": "true",
        "package_root": "packages/app",
        "ts_package_manager": "pnpm",
        "ts_pnpm_version": "10.4.1",
        "python_env": "pip",
        "provision_rust": "true",
        "cargo_target_dir": "packages/app/target",
        "config": "packages/app/testing-conventions.toml",
        "build_command": "pnpm build",
        "packaging_build": "pnpm pack --pack-destination dist",
        "packaging_language": "typescript",
        "e2e_extra_scope": "--extra-scope core",
        "e2e_exclude": "--exclude core/cli",
        "cli_command": "./hermetic-cli/testing-conventions",
        "ts_mutation_adapter_args": "--ts-mutation-adapter ./hermetic-cli/dist/mutation/main.js",
    }


def test_compute_outputs_takes_the_published_path_for_another_caller(tmp_path, in_dir):
    in_dir(tmp_path)
    build_ts_package(tmp_path / "packages" / "app")
    outputs = detect.compute_outputs("", "packages/app/src", caller_repository="acme/app")
    assert (outputs["cli_command"], outputs["ts_mutation_adapter_args"]) == ("", "")


RUST_BEARING = (
    "colocated_test_languages",
    "integration_lint_languages",
    "isolation_languages",
    "static_languages",
    "one_function_languages",
    "coverage_languages",
    "mutation_languages",
)


def test_compute_outputs_keeps_rust_out_of_the_suite_matrix(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "crate" / "Cargo.toml", '[package]\nname = "c"\n')
    write(tmp_path / "crate" / "src" / "lib.rs", "pub fn f() {}\n")
    write(tmp_path / "crate" / "mod.py")
    outputs = detect.compute_outputs("", "crate")
    assert outputs["languages"] == '["python"]'
    assert [outputs[name] for name in RUST_BEARING] == ['["python","rust"]'] * len(RUST_BEARING)


def test_compute_outputs_honors_a_restricted_language_list(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "crate" / "Cargo.toml", '[package]\nname = "c"\n')
    write(tmp_path / "crate" / "src" / "lib.rs", "pub fn f() {}\n")
    write(tmp_path / "crate" / "mod.py")
    outputs = detect.compute_outputs('["python"]', "crate")
    assert [outputs[name] for name in RUST_BEARING] == ['["python"]'] * len(RUST_BEARING)


def test_compute_outputs_names_no_packaging_language_without_a_build(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "src" / "mod.py")
    outputs = detect.compute_outputs("", "src")
    assert (outputs["packaging_build"], outputs["packaging_language"]) == ("", "")
    assert (outputs["packaging_dist"], outputs["e2e_attestation"]) == ("false", "false")


def test_compute_outputs_keys_the_cargo_cache_on_the_workspace_root(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "ws" / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    member = tmp_path / "ws" / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    write(member / "src" / "lib.rs", "pub fn f() {}\n")
    outputs = detect.compute_outputs("", "ws/crates/core")
    assert (outputs["package_root"], outputs["cargo_target_dir"]) == ("ws/crates/core", "ws/target")


def test_compute_outputs_reports_a_package_root_outside_the_repo_as_the_repo_root(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "outside" / "package.json", "{}")
    (tmp_path / "repo").mkdir()
    outputs = detect.compute_outputs("", str(tmp_path / "outside"), repo_root=str(tmp_path / "repo"))
    assert (outputs["package_root"], outputs["cargo_target_dir"]) == (".", "./target")


def test_compute_outputs_reports_a_workspace_outside_the_repo_as_the_repo_root(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "ws" / "Cargo.toml", '[workspace]\n')
    member = tmp_path / "ws" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    (tmp_path / "repo").mkdir()
    outputs = detect.compute_outputs("", str(member), repo_root=str(tmp_path / "repo"))
    assert outputs["cargo_target_dir"] == "./target"


def run_main(env):
    """Run `main` with `env` as the whole environment."""
    with patch.dict(os.environ, env, clear=True):
        return detect.main()


def test_main_scans_the_working_directory_by_default(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    assert run_main({}) == 0
    assert capsys.readouterr().out.startswith("languages='' under '.' -> languages [\"python\"], ")


def test_main_scans_the_requested_path(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.ts")
    write(tmp_path / "pkg" / "mod.py")
    run_main({"SCAN_PATH": "pkg"})
    assert capsys.readouterr().out.startswith("languages='' under 'pkg' -> languages [\"python\"], ")


def test_main_restricts_the_scan_to_the_requested_languages(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main({"LANGUAGES": '["typescript"]'})
    assert capsys.readouterr().out.startswith("languages='[\"typescript\"]' under '.' -> languages [], ")


def test_main_reads_the_requested_config(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    write(tmp_path / "custom.toml", '[python]\nbuild_command = "make"\n')
    write(tmp_path / "mod.py")
    run_main({"CONFIG": "custom.toml"})
    assert "config custom.toml, build_command make," in capsys.readouterr().out


def test_main_builds_the_cli_from_head_for_this_repositorys_own_run(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main({"CALLER_REPOSITORY": "thekevinscott/testing-conventions"})
    assert "cli_command ./hermetic-cli/testing-conventions," in capsys.readouterr().out


def test_main_takes_the_published_path_when_a_version_is_requested(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    env = {"CALLER_REPOSITORY": "thekevinscott/testing-conventions", "VERSION": "1.2.3"}
    run_main(env)
    assert "cli_command , " in capsys.readouterr().out


def test_main_appends_the_outputs_to_the_github_output_file(tmp_path, in_dir):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    receipt = write(tmp_path / "outputs.txt", "already=here\n")
    run_main({"GITHUB_OUTPUT": str(receipt)})
    written = receipt.read_text()
    assert written.startswith("already=here\n")
    assert 'languages=["python"]\n' in written
    assert written.endswith("ts_mutation_adapter_args=\n")


def test_main_prints_the_outputs_with_no_github_output_file(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main({})
    assert capsys.readouterr().out.endswith(", cli_command , ts_mutation_adapter_args \n")


def test_running_the_module_as_a_script_exits_with_mains_status(tmp_path, in_dir):
    in_dir(tmp_path)
    run_name = "".join(["__main", "__"])
    with patch.dict(os.environ, {}, clear=True):
        with pytest.raises(SystemExit) as exit_info:
            runpy.run_path(detect.__file__, run_name=run_name)
    assert exit_info.value.code == 0


@pytest.mark.parametrize("run_name", ["__init__", "detect"])
def test_running_the_module_under_any_other_name_leaves_main_uncalled(tmp_path, in_dir, run_name):
    in_dir(tmp_path)
    with patch.dict(os.environ, {}, clear=True):
        assert runpy.run_path(detect.__file__, run_name=run_name)["__name__"] == run_name
