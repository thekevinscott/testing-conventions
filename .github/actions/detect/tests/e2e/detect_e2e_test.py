"""End-to-end tests for the detect script: a real directory tree, no mocks.

Per the standard, an e2e test runs with no mocks. The `run_detect` fixture builds a real scan tree
(and, at the checkout root, an optional `dist/` + attestation), then runs the script's `__main__`
entry point in-process via `runpy` with `LANGUAGES` / `SCAN_PATH` / `GITHUB_OUTPUT` in the env —
the inputs the composite action passes — and parses the `name=value` lines it writes. Running the
real entry point in-process keeps the filesystem boundary and the `__main__` guard on the
measured-coverage path; the env is set with `patch.dict` and the working directory is confined to
the fixture.
"""
import os
import runpy
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
import detect  # noqa: E402

SCRIPT = Path(__file__).resolve().parents[2] / "detect.py"


@pytest.fixture
def run_detect(tmp_path):
    """A `run(...) -> {output: value}` that runs detect.py as `__main__` over a built tree."""
    origin_cwd = os.getcwd()
    os.chdir(tmp_path)

    def run(
        languages="",
        sources=None,
        root_files=None,
        github_output="github_output",
        scan_path="scan",
        config="testing-conventions.toml",
    ):
        scan = Path(scan_path)
        scan.mkdir(parents=True, exist_ok=True)
        for rel, content in (sources or {}).items():
            path = scan / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        for rel, content in (root_files or {}).items():  # relative to the checkout root (cwd)
            path = Path(rel)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        out_path = Path(github_output) if github_output else None
        if out_path:
            out_path.write_text("")
        env = {
            "LANGUAGES": languages,
            "SCAN_PATH": scan_path,
            "GITHUB_OUTPUT": github_output,
            "CONFIG": config,
        }
        with patch.dict(os.environ, env):
            try:
                runpy.run_path(str(SCRIPT), run_name="__main__")
            except SystemExit:
                pass
        if not out_path:
            return {}
        return dict(
            line.split("=", 1)
            for line in out_path.read_text().splitlines() if "=" in line
        )

    try:
        yield run
    finally:
        os.chdir(origin_cwd)


def test_e2e_explicit_python(run_detect):
    out = run_detect(languages='["python"]', sources={"widget.py": "x = 1\n"})
    assert out["languages"] == '["python"]'
    assert out["coverage_languages"] == '["python"]'


def test_e2e_auto_detects_a_rust_crate(run_detect):
    out = run_detect(sources={"Cargo.toml": '[package]\nname = "x"\n', "src/lib.rs": "pub fn f() {}\n"})
    assert '"rust"' in out["integration_lint_languages"]
    assert out["coverage_languages"] == '["rust"]'


def test_e2e_rust_crate_enters_the_colocated_test_matrix(run_detect):
    # #274: the whole-tree colocated-test matrix carries rust (inline `#[cfg(test)]`
    # presence, #40); the co-change matrix (`languages`) stays python/typescript.
    out = run_detect(sources={"Cargo.toml": '[package]\nname = "x"\n', "src/lib.rs": "pub fn f() {}\n"})
    assert out["colocated_test_languages"] == '["rust"]'
    assert out["languages"] == "[]"


def test_e2e_cargo_without_rust_source_is_not_a_crate(run_detect):
    out = run_detect(sources={"Cargo.toml": '[package]\nname = "x"\n'})
    assert out["coverage_languages"] == "[]"
    assert out["integration_lint_languages"] == "[]"


def test_e2e_absent_language_is_skipped(run_detect):
    out = run_detect(languages='["python","typescript"]', sources={"widget.py": "x = 1\n"})
    assert out["languages"] == '["python"]'


def test_e2e_packaging_dist_located(run_detect):
    out = run_detect(root_files={"dist/widget-0.1.0-py3-none-any.whl": ""})
    assert out["packaging_dist"] == "true"


# --- #280: packaging_dist is looked for at the derived package root, not the checkout root ---


def test_e2e_packaging_dist_found_at_the_derived_package_root(run_detect):
    out = run_detect(
        scan_path="packages/x/src",
        root_files={
            "packages/x/package.json": "{}",
            "packages/x/src/index.ts": "export const x = 1;\n",
            "packages/x/dist/pkg.tgz": "",
        },
    )
    assert out["package_root"] == "packages/x"
    assert out["packaging_dist"] == "true"


def test_e2e_packaging_dist_at_the_repo_root_is_not_found_for_a_scoped_package(run_detect):
    out = run_detect(
        scan_path="packages/x/src",
        root_files={
            "packages/x/package.json": "{}",
            "packages/x/src/index.ts": "export const x = 1;\n",
            "dist/pkg.tgz": "",  # at the checkout root, not the package's own dist/
        },
    )
    assert out["package_root"] == "packages/x"
    assert out["packaging_dist"] == "false"


def test_e2e_packaging_dist_at_the_repo_root_still_found_for_a_single_package_repo(run_detect):
    # Regression guard: no manifest above the scan root derives package_root == "." (the
    # checkout root), so a root-level dist/ is unchanged from today's behavior.
    out = run_detect(root_files={"dist/widget-0.1.0-py3-none-any.whl": ""})
    assert out["package_root"] == "."
    assert out["packaging_dist"] == "true"


def test_e2e_attestation_detected(run_detect):
    out = run_detect(root_files={"e2e-attestation.json": "{}"})
    assert out["e2e_attestation"] == "true"
    assert out["packaging_dist"] == "false"  # no dist/ alongside


def test_e2e_runs_without_a_github_output_file(run_detect, capsys):
    # GITHUB_OUTPUT empty: the script still runs and prints a summary, writing no output file.
    out = run_detect(languages='["python"]', sources={"widget.py": "x = 1\n"}, github_output="")
    assert out == {}
    assert "languages" in capsys.readouterr().out


# --- #277: the monorepo package-root primitive ---


def test_e2e_package_root_at_nested_manifest(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": "{}",
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["package_root"] == "packages/ts"


def test_e2e_package_root_equals_scan_root_when_the_manifest_is_there(run_detect):
    out = run_detect(
        scan_path="packages/rs",
        root_files={
            "packages/rs/Cargo.toml": '[package]\nname = "x"\n',
            "packages/rs/lib.rs": "pub fn f() {}\n",
        },
    )
    assert out["package_root"] == "packages/rs"


def test_e2e_package_root_falls_back_to_the_repo_root(run_detect):
    out = run_detect(sources={"widget.py": "x = 1\n"})
    assert out["package_root"] == "."


def test_e2e_ts_package_manager_from_packagemanager_field(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"packageManager": "pnpm@8.6.0"}',
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["ts_package_manager"] == "pnpm"


def test_e2e_ts_package_manager_field_beats_lockfile(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"packageManager": "npm@10.0.0"}',
            "packages/ts/pnpm-lock.yaml": "",
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["ts_package_manager"] == "npm"


def test_e2e_ts_package_manager_from_npm_lockfile(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": "{}",
            "packages/ts/package-lock.json": "{}",
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["ts_package_manager"] == "npm"


def test_e2e_ts_package_manager_defaults_to_pnpm(run_detect):
    out = run_detect(sources={"widget.ts": "export const x = 1;\n"})
    assert out["ts_package_manager"] == "pnpm"


def test_e2e_ts_package_manager_pnpm_lockfile_with_no_field(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": "{}",
            "packages/ts/pnpm-lock.yaml": "",
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["ts_package_manager"] == "pnpm"


def test_e2e_read_package_json_falls_back_to_empty_on_malformed_json(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": "not valid json {{{",
            "packages/ts/package-lock.json": "{}",
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    # A malformed package.json never crashes detect: no packageManager field is readable,
    # so ts_package_manager falls through to the lockfile tier.
    assert out["ts_package_manager"] == "npm"


def test_e2e_python_env_uv_when_project_table_present(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\nversion = "0.1.0"\n',
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["python_env"] == "uv"


def test_e2e_python_env_pip_without_a_project_table(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": "[tool.black]\nline-length = 100\n",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["python_env"] == "pip"


def test_e2e_python_env_pip_without_a_pyproject(run_detect):
    out = run_detect(sources={"widget.py": "x = 1\n"})
    assert out["python_env"] == "pip"


def test_e2e_python_env_pip_on_an_unparseable_pyproject(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": "not valid toml [[[",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["python_env"] == "pip"


def test_e2e_provision_rust_true_for_a_cargo_toml_at_the_package_root(run_detect):
    out = run_detect(
        scan_path="packages/rs/src",
        root_files={
            "packages/rs/Cargo.toml": '[package]\nname = "x"\n',
            "packages/rs/src/lib.rs": "pub fn f() {}\n",
        },
    )
    assert out["provision_rust"] == "true"


def test_e2e_provision_rust_true_for_a_maturin_backend(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": (
                '[project]\nname = "x"\n\n[build-system]\nbuild-backend = "maturin"\n'
            ),
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["provision_rust"] == "true"


def test_e2e_provision_rust_true_for_a_napi_key(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"napi": {}}',
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["provision_rust"] == "true"


def test_e2e_provision_rust_true_for_a_napi_cli_dev_dependency(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"devDependencies": {"@napi-rs/cli": "^2.0.0"}}',
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["provision_rust"] == "true"


def test_e2e_provision_rust_false_by_default(run_detect):
    out = run_detect(sources={"widget.py": "x = 1\n"})
    assert out["provision_rust"] == "false"


def test_derive_package_root_falls_back_to_repo_root_when_scan_root_is_unrelated(tmp_path_factory):
    # scan_root and repo_root live in disjoint trees, so walking up from scan_root never
    # reaches repo_root — the walk exhausts at the filesystem root, and repo_root is appended
    # as the final fallback candidate rather than already being one from the walk.
    scan_root = tmp_path_factory.mktemp("scan-tree")
    repo_root = tmp_path_factory.mktemp("repo-tree")
    assert detect.derive_package_root(scan_root, repo_root) == repo_root.resolve()


def test_derive_package_root_never_searches_outside_repo_root(tmp_path_factory):
    # A manifest sitting *above* repo_root (outside the checkout) must never be returned: the
    # walk stops at repo_root, inclusive, even though repo_root itself carries no manifest here.
    base = tmp_path_factory.mktemp("outside-base")
    (base / "Cargo.toml").write_text('[package]\nname = "outside"\n')
    repo_root = base / "repo"
    scan_root = repo_root / "src"
    scan_root.mkdir(parents=True)
    assert detect.derive_package_root(scan_root, repo_root) == repo_root.resolve()


def test_derive_package_root_boundary_is_an_exact_match_not_an_ordering(tmp_path):
    # A regression guard against a specific mutation-testing trap: the walk's stop condition
    # (`ancestor == repo_root`) must be an exact match, never an ordering comparison. `repo_root`
    # here is a disjoint sibling that sorts lexicographically *after* scan_root's own ancestor
    # chain, so a `<=` in place of `==` would treat scan_root as already "past" repo_root and
    # stop the walk on the very first candidate — before ever climbing to `base`, which is a
    # real ancestor of scan_root carrying a manifest an `==`-based walk correctly finds.
    base = tmp_path / "aaa"
    scan_root = base / "pkg" / "src"
    scan_root.mkdir(parents=True)
    (base / "Cargo.toml").write_text('[package]\nname = "x"\n')
    repo_root = tmp_path / "zzz"
    repo_root.mkdir()
    assert scan_root.resolve() <= repo_root.resolve()  # pins the ordering this test relies on
    assert detect.derive_package_root(scan_root, repo_root) == base.resolve()


def test_e2e_config_default_falls_back_when_no_package_root_file(run_detect):
    out = run_detect(sources={"widget.py": "x = 1\n"})
    assert out["config"] == "testing-conventions.toml"


def test_e2e_config_default_discovers_the_package_root_file(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["config"] == "packages/py/testing-conventions.toml"


def test_e2e_config_explicit_override_wins_verbatim(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        config="custom.toml",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    # A caller-provided non-default value passes through unchanged even though a
    # package-root file exists, since the explicit override always wins.
    assert out["config"] == "custom.toml"


def test_e2e_config_explicit_override_sorts_after_the_default_lexicographically(run_detect):
    # "zzz-custom.toml" sorts after "testing-conventions.toml", unlike "custom.toml" above
    # (which sorts before it) — together they pin the comparison to inequality, not ordering.
    out = run_detect(
        scan_path="packages/py/src",
        config="zzz-custom.toml",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["config"] == "zzz-custom.toml"


# --- #281: e2e-verify attestation discovery scoped to the package root ---


def test_e2e_attestation_at_the_package_root_is_detected(run_detect):
    out = run_detect(
        scan_path="packages/x/src",
        root_files={
            "packages/x/package.json": "{}",
            "packages/x/src/widget.ts": "export const x = 1;\n",
            "packages/x/e2e-attestation.json": "{}",
        },
    )
    assert out["e2e_attestation"] == "true"


def test_e2e_attestation_at_the_repo_root_is_not_detected_for_a_nested_package(run_detect):
    # The attestation moved from repo-root lookup to package-root lookup (#281): a
    # repo-root attestation no longer counts for a scan scoped to a nested package.
    out = run_detect(
        scan_path="packages/x/src",
        root_files={
            "packages/x/package.json": "{}",
            "packages/x/src/widget.ts": "export const x = 1;\n",
            "e2e-attestation.json": "{}",
        },
    )
    assert out["e2e_attestation"] == "false"


def test_e2e_attestation_at_the_repo_root_is_still_detected_for_a_single_package_repo(run_detect):
    # Regression guard: a single-package repo (no manifest above `scan_path`) still
    # derives `package_root == repo_root`, so a repo-root attestation is unchanged.
    out = run_detect(
        scan_path="src",
        root_files={
            "src/widget.py": "x = 1\n",
            "e2e-attestation.json": "{}",
        },
    )
    assert out["e2e_attestation"] == "true"


# --- #289: the [python].build_command escape hatch, read from the package's own config ---


def test_e2e_build_command_derived_from_the_package_root_config(run_detect):
    # The escape hatch moves from a `uses:`-call input to a `[python] build_command` key in
    # the package's own testing-conventions.toml, discovered at the package root exactly like
    # `config` itself (never passed on the call). `detect` opens that file and emits it.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": (
                '[python]\nbuild_command = "uv run maturin develop"\n'
                'reason = "maturin\'s PEP 517 backend has no pre-build shell hook"\n'
            ),
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["config"] == "packages/py/testing-conventions.toml"
    assert out["build_command"] == "uv run maturin develop"


def test_e2e_build_command_from_an_explicit_config_override(run_detect):
    # An explicit non-default `config` path is honored verbatim (like `config` today), and
    # build_command is read from that same file.
    out = run_detect(
        scan_path="packages/py/src",
        config="packages/py/custom.toml",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/custom.toml": (
                '[python]\nbuild_command = "pnpm build"\n'
                'reason = "the addon is built by a workspace script"\n'
            ),
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["build_command"] == "pnpm build"


def test_e2e_build_command_absent_is_empty(run_detect):
    # No config file at all: byte-identical to the old empty `build_command: ''` default —
    # no build step.
    out = run_detect(sources={"widget.py": "x = 1\n"})
    assert out["build_command"] == ""


def test_e2e_build_command_empty_when_config_declares_none(run_detect):
    # A package-root config with a [python] table but no build_command emits an empty
    # build_command.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "[python]\ncoverage = { fail_under = 90 }\n",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["build_command"] == ""


def test_e2e_build_command_empty_when_config_has_no_python_table(run_detect):
    # A config with no [python] table at all (a rust-only config) emits an empty build_command.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "[rust]\nfeatures = [\"cli\"]\n",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["build_command"] == ""


def test_e2e_build_command_empty_on_a_malformed_config(run_detect):
    # A malformed testing-conventions.toml never crashes detect — build_command falls back to
    # empty, like read_pyproject on a malformed manifest.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "not valid toml [[[",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["build_command"] == ""


def test_e2e_build_command_empty_when_value_is_not_a_string(run_detect):
    # A non-string build_command (which the Rust config loader would separately reject) is
    # treated as absent by detect rather than emitted verbatim — detect never crashes on it.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "[python]\nbuild_command = 123\n",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["build_command"] == ""


# --- #333: the [e2e] extra_scope / exclude freshness roots, read from the package's own config ---


def test_e2e_extra_scope_and_exclude_rendered_as_repeated_flags(run_detect):
    # A binding package declares the shared core beside it as an extra freshness root, with the
    # feature-gated cli/ and bin/ excluded. detect discovers the config at the package root
    # (like `config`/`build_command`) and renders repeated --extra-scope/--exclude arguments the
    # e2e-verify run step appends verbatim.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": (
                '[e2e]\nextra_scope = ["packages/rust/src"]\n'
                'exclude = ["packages/rust/src/cli", "packages/rust/src/bin"]\n'
            ),
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["config"] == "packages/py/testing-conventions.toml"
    assert out["e2e_extra_scope"] == "--extra-scope packages/rust/src"
    assert out["e2e_exclude"] == "--exclude packages/rust/src/cli --exclude packages/rust/src/bin"


def test_e2e_extra_scope_and_exclude_absent_is_empty(run_detect):
    # No config file at all: byte-identical to before — no extra roots, no excludes.
    out = run_detect(sources={"widget.py": "x = 1\n"})
    assert out["e2e_extra_scope"] == ""
    assert out["e2e_exclude"] == ""


def test_e2e_extra_scope_empty_when_config_declares_no_e2e_table(run_detect):
    # A package-root config with no [e2e] table emits empty extra-scope/exclude.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "[python]\ncoverage = { fail_under = 90 }\n",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["e2e_extra_scope"] == ""
    assert out["e2e_exclude"] == ""


def test_e2e_extra_scope_empty_on_a_malformed_config(run_detect):
    # A malformed testing-conventions.toml never crashes detect — extra-scope falls back to empty.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "not valid toml [[[",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["e2e_extra_scope"] == ""


def test_e2e_extra_scope_empty_when_value_is_not_a_list(run_detect):
    # A non-list extra_scope (which the tool would separately reject) is treated as absent by
    # detect rather than emitted — detect never crashes on it.
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": '[e2e]\nextra_scope = "packages/rust/src"\n',
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["e2e_extra_scope"] == ""


def test_e2e_extra_scope_skips_blank_and_non_string_entries(run_detect):
    # A blank string renders as an empty `--extra-scope ` argument and a non-string can't render
    # at all, so both are skipped — detect never emits a malformed flag. (The config loader
    # separately rejects a non-string, but a blank string is a valid `Vec<String>` entry it
    # accepts, so the guard earns its keep.)
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": '[e2e]\nextra_scope = ["packages/rust/src", "", 5]\n',
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["e2e_extra_scope"] == "--extra-scope packages/rust/src"
