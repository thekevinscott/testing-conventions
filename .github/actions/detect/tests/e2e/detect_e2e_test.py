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
