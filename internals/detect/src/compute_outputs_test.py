from pathlib import Path

import compute_outputs


def test_to_json_is_compact():
    assert compute_outputs._to_json(["python", "rust"]) == '["python","rust"]'


def test_to_json_empty_is_brackets():
    assert compute_outputs._to_json([]) == "[]"


def build_ts_package(root: Path, write) -> None:
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


def test_compute_outputs_derives_every_output_from_the_package(tmp_path, in_dir, write):
    in_dir(tmp_path)
    build_ts_package(tmp_path / "packages" / "app", write)
    assert compute_outputs.compute_outputs(
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


def test_compute_outputs_takes_the_published_path_for_another_caller(tmp_path, in_dir, write):
    in_dir(tmp_path)
    build_ts_package(tmp_path / "packages" / "app", write)
    outputs = compute_outputs.compute_outputs("", "packages/app/src", caller_repository="acme/app")
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


def test_compute_outputs_keeps_rust_out_of_the_suite_matrix(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "crate" / "Cargo.toml", '[package]\nname = "c"\n')
    write(tmp_path / "crate" / "src" / "lib.rs", "pub fn f() {}\n")
    write(tmp_path / "crate" / "mod.py")
    outputs = compute_outputs.compute_outputs("", "crate")
    assert outputs["languages"] == '["python"]'
    assert [outputs[name] for name in RUST_BEARING] == ['["python","rust"]'] * len(RUST_BEARING)


def test_compute_outputs_honors_a_restricted_language_list(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "crate" / "Cargo.toml", '[package]\nname = "c"\n')
    write(tmp_path / "crate" / "src" / "lib.rs", "pub fn f() {}\n")
    write(tmp_path / "crate" / "mod.py")
    outputs = compute_outputs.compute_outputs('["python"]', "crate")
    assert [outputs[name] for name in RUST_BEARING] == ['["python"]'] * len(RUST_BEARING)


def test_compute_outputs_names_no_packaging_language_without_a_build(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "src" / "mod.py")
    outputs = compute_outputs.compute_outputs("", "src")
    assert (outputs["packaging_build"], outputs["packaging_language"]) == ("", "")
    assert (outputs["packaging_dist"], outputs["e2e_attestation"]) == ("false", "false")


def test_compute_outputs_keys_the_cargo_cache_on_the_workspace_root(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "ws" / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    member = tmp_path / "ws" / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    write(member / "src" / "lib.rs", "pub fn f() {}\n")
    outputs = compute_outputs.compute_outputs("", "ws/crates/core")
    assert (outputs["package_root"], outputs["cargo_target_dir"]) == ("ws/crates/core", "ws/target")


def test_compute_outputs_reports_a_package_root_outside_the_repo_as_the_repo_root(
    tmp_path, in_dir, write
):
    in_dir(tmp_path)
    write(tmp_path / "outside" / "package.json", "{}")
    (tmp_path / "repo").mkdir()
    outputs = compute_outputs.compute_outputs(
        "", str(tmp_path / "outside"), repo_root=str(tmp_path / "repo")
    )
    assert (outputs["package_root"], outputs["cargo_target_dir"]) == (".", "./target")


def test_compute_outputs_reports_a_workspace_outside_the_repo_as_the_repo_root(
    tmp_path, in_dir, write
):
    in_dir(tmp_path)
    write(tmp_path / "ws" / "Cargo.toml", '[workspace]\n')
    member = tmp_path / "ws" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    (tmp_path / "repo").mkdir()
    outputs = compute_outputs.compute_outputs("", str(member), repo_root=str(tmp_path / "repo"))
    assert outputs["cargo_target_dir"] == "./target"
