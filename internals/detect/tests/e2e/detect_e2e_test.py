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
import re
import runpy
from pathlib import Path
from unittest.mock import patch

import pytest

import detect

SCRIPT = Path(__file__).resolve().parents[2] / "src" / "detect.py"
REPO_ROOT = Path(__file__).resolve().parents[4]
ACTION_YML = REPO_ROOT / ".github" / "actions" / "detect" / "action.yml"
WORKFLOW_YML = REPO_ROOT / ".github" / "workflows" / "testing-conventions.yml"


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
        caller_repository="",
        version="",
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
            "CALLER_REPOSITORY": caller_repository,
            "VERSION": version,
        }
        with patch.dict(os.environ, env):
            try:
                runpy.run_path(str(SCRIPT), run_name="__main__")
            except SystemExit:
                pass
        if not out_path:
            return {}
        return _parse_output_file(out_path.read_text())

    try:
        yield run
    finally:
        os.chdir(origin_cwd)


def _parse_output_file(text):
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


def _declared_outputs():
    """The composite action's `outputs:` block, as `name -> value expression`.

    Parsed with the stdlib against the manifest's fixed two-space shape: `detect.py` is
    stdlib-only by contract, so its test package carries pytest and nothing else.
    """
    lines = ACTION_YML.read_text().split("\n")
    start = lines.index("outputs:") + 1
    declared, name = {}, None
    for line in lines[start:]:
        if line and not line.startswith(" "):  # the next top-level key ends the block
            break
        header = re.match(r"^  ([A-Za-z_][A-Za-z0-9_]*):", line)
        if header:
            name = header.group(1)
            declared[name] = ""
            continue
        value = re.match(r"^    value: (.*)$", line)
        if value and name is not None:
            declared[name] = value.group(1).strip()
    return declared


def _detect_job_outputs(text):
    """The reusable workflow `detect` job's `outputs:` block, as `name -> value expression`.

    `[a-z0-9_]+`, never `[a-z_]+`: three output names carry a digit (`e2e_attestation`,
    `e2e_extra_scope`, `e2e_exclude`), and a pattern that drops them drops them from both sides
    of a set comparison at once, leaving the equality true over a smaller set.
    """
    lines = text.split("\n")
    start = lines.index("    outputs:", lines.index("  detect:")) + 1
    outputs = {}
    for line in lines[start:]:
        entry = re.match(r"^      ([a-z0-9_]+): (.*)$", line)
        if not entry:  # the next job-level key ends the block
            break
        outputs[entry.group(1)] = entry.group(2).strip()
    return outputs


def _detect_references(text):
    """Every `needs.detect.outputs.<name>` a job reads."""
    return set(re.findall(r"needs\.detect\.outputs\.([a-z0-9_]+)", text))


def test_e2e_explicit_python(run_detect):
    out = run_detect(languages='["python"]', sources={"widget.py": "x = 1\n"})
    assert out["languages"] == '["python"]'
    assert out["coverage_languages"] == '["python"]'


def test_e2e_auto_detects_a_rust_crate(run_detect):
    out = run_detect(sources={"Cargo.toml": '[package]\nname = "x"\n', "src/lib.rs": "pub fn f() {}\n"})
    assert '"rust"' in out["integration_lint_languages"]
    assert out["coverage_languages"] == '["rust"]'


def test_e2e_rust_crate_enters_the_colocated_test_matrix(run_detect):
    out = run_detect(sources={"Cargo.toml": '[package]\nname = "x"\n', "src/lib.rs": "pub fn f() {}\n"})
    assert out["colocated_test_languages"] == '["rust"]'
    assert out["languages"] == "[]"


def test_e2e_one_function_matrix_carries_every_detected_language(run_detect):
    out = run_detect(
        sources={
            "widget.py": "x = 1\n",
            "Cargo.toml": '[package]\nname = "x"\n',
            "src/lib.rs": "pub fn f() {}\n",
        }
    )
    assert out["one_function_languages"] == '["python","rust"]'


def test_e2e_one_function_matrix_is_empty_on_an_empty_tree(run_detect):
    out = run_detect()
    assert out["one_function_languages"] == "[]"


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
    out = run_detect(root_files={"dist/widget-0.1.0-py3-none-any.whl": ""})
    assert out["package_root"] == "."
    assert out["packaging_dist"] == "true"


def test_e2e_attestation_detected(run_detect):
    out = run_detect(root_files={"e2e-attestation.json": "{}"})
    assert out["e2e_attestation"] == "true"
    assert out["packaging_dist"] == "false"  # no dist/ alongside


def test_e2e_attestation_receipts_directory_detected(run_detect):
    out = run_detect(root_files={"e2e-attestations/feature-one-abcd012345.json": "{}"})
    assert out["e2e_attestation"] == "true"


def test_e2e_attestation_receipts_directory_without_receipts_is_not_detected(run_detect):
    out = run_detect(root_files={"e2e-attestations/README.md": ""})
    assert out["e2e_attestation"] == "false"


def test_e2e_runs_without_a_github_output_file(run_detect, capsys):
    out = run_detect(languages='["python"]', sources={"widget.py": "x = 1\n"}, github_output="")
    assert out == {}
    assert "languages" in capsys.readouterr().out


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


def test_e2e_ts_pnpm_version_echoes_a_packagemanager_pnpm_pin(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"packageManager": "pnpm@10.33.0"}',
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["ts_pnpm_version"] == "10.33.0"


def test_e2e_ts_pnpm_version_is_never_empty_for_a_pnpm_pin(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"packageManager": "pnpm@10.33.0+sha512.abc123"}',
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["ts_pnpm_version"] == "10.33.0+sha512.abc123"


def test_e2e_ts_pnpm_version_is_the_floor_with_no_packagemanager_field(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": "{}",
            "packages/ts/pnpm-lock.yaml": "",
            "packages/ts/src/index.ts": "export const x = 1;\n",
        },
    )
    assert out["ts_pnpm_version"] == ">=11"


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
    scan_root = tmp_path_factory.mktemp("scan-tree")
    repo_root = tmp_path_factory.mktemp("repo-tree")
    assert detect.derive_package_root(scan_root, repo_root) == repo_root.resolve()


def test_derive_package_root_never_searches_outside_repo_root(tmp_path_factory):
    base = tmp_path_factory.mktemp("outside-base")
    (base / "Cargo.toml").write_text('[package]\nname = "outside"\n')
    repo_root = base / "repo"
    scan_root = repo_root / "src"
    scan_root.mkdir(parents=True)
    assert detect.derive_package_root(scan_root, repo_root) == repo_root.resolve()


def test_derive_package_root_boundary_is_an_exact_match_not_an_ordering(tmp_path):
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
    assert out["config"] == "custom.toml"


def test_e2e_config_explicit_override_sorts_after_the_default_lexicographically(run_detect):
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
    out = run_detect(
        scan_path="src",
        root_files={
            "src/widget.py": "x = 1\n",
            "e2e-attestation.json": "{}",
        },
    )
    assert out["e2e_attestation"] == "true"


def test_e2e_build_command_derived_from_the_package_root_config(run_detect):
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


def test_e2e_multiline_build_command_round_trips_through_github_output(run_detect):
    build = "cp a.tmpl a.py\ncp b.tmpl b.py"
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": (
                '[python]\nbuild_command = """cp a.tmpl a.py\ncp b.tmpl b.py"""\n'
                'reason = "the backend has no pre-build shell hook"\n'
            ),
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["build_command"] == build


def test_e2e_build_command_from_an_explicit_config_override(run_detect):
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
    out = run_detect(sources={"widget.py": "x = 1\n"})
    assert out["build_command"] == ""


def test_e2e_build_command_empty_when_config_declares_none(run_detect):
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
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": "[python]\nbuild_command = 123\n",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["build_command"] == ""


def test_e2e_build_command_derived_for_a_manifest_less_pip_python_package(run_detect):
    out = run_detect(
        languages='["python"]',
        root_files={
            "testing-conventions.toml": (
                '[python]\nbuild_command = "cp generated.py.tmpl generated.py"\n'
            ),
        },
        sources={"widget.py": "from generated import OFFSET\n"},
    )
    assert out["build_command"] == "cp generated.py.tmpl generated.py"


def test_e2e_build_command_empty_when_manifest_less_and_ambiguous(run_detect):
    out = run_detect(
        languages='["python","typescript"]',
        root_files={
            "testing-conventions.toml": '[python]\nbuild_command = "cp a.tmpl a.py"\n',
        },
        sources={"widget.py": "x = 1\n", "index.ts": "export const x = 1;\n"},
    )
    assert out["build_command"] == ""


def test_e2e_extra_scope_and_exclude_rendered_as_repeated_flags(run_detect):
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
    out = run_detect(sources={"widget.py": "x = 1\n"})
    assert out["e2e_extra_scope"] == ""
    assert out["e2e_exclude"] == ""


def test_e2e_extra_scope_empty_when_config_declares_no_e2e_table(run_detect):
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
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/testing-conventions.toml": '[e2e]\nextra_scope = ["packages/rust/src", "", 5]\n',
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["e2e_extra_scope"] == "--extra-scope packages/rust/src"


def test_e2e_packaging_build_is_uv_build_for_a_python_project(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["packaging_build"] == "uv build"
    assert out["packaging_language"] == "python"


def test_e2e_packaging_build_is_pnpm_pack_for_a_pnpm_package(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"name": "x"}\n',
            "packages/ts/pnpm-lock.yaml": "lockfileVersion: '9.0'\n",
            "packages/ts/src/widget.ts": "export const x = 1;\n",
        },
    )
    assert out["packaging_build"] == "pnpm pack --pack-destination dist"
    assert out["packaging_language"] == "typescript"


def test_e2e_packaging_build_is_npm_pack_for_an_npm_package(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"name": "x"}\n',
            "packages/ts/package-lock.json": "{}\n",
            "packages/ts/src/widget.ts": "export const x = 1;\n",
        },
    )
    assert out["packaging_build"] == "npm pack --pack-destination dist"


def test_e2e_packaging_build_is_cargo_package_for_a_crate(run_detect):
    out = run_detect(
        sources={"Cargo.toml": '[package]\nname = "x"\n', "src/lib.rs": "pub fn f() {}\n"},
    )
    assert out["packaging_build"] == "cargo package"
    assert out["packaging_language"] == "rust"


def test_e2e_packaging_build_redirects_target_dir_for_a_workspace_member(run_detect):
    out = run_detect(
        scan_path="packages/rust/src",
        root_files={
            "Cargo.toml": '[workspace]\nmembers = ["packages/rust"]\n',
            "packages/rust/Cargo.toml": '[package]\nname = "x"\n',
            "packages/rust/src/lib.rs": "pub fn f() {}\n",
        },
    )
    assert out["packaging_build"] == "cargo package --target-dir target"
    assert out["packaging_language"] == "rust"


def test_e2e_packaging_build_unredirected_for_a_standalone_crate_with_no_workspace(run_detect):
    out = run_detect(
        sources={"Cargo.toml": '[package]\nname = "x"\n', "src/lib.rs": "pub fn f() {}\n"},
    )
    assert out["packaging_build"] == "cargo package"


def test_e2e_packaging_build_unredirected_for_a_crate_that_is_itself_the_workspace_root(run_detect):
    out = run_detect(
        sources={
            "Cargo.toml": '[package]\nname = "x"\n\n[workspace]\nmembers = ["."]\n',
            "src/lib.rs": "pub fn f() {}\n",
        },
    )
    assert out["packaging_build"] == "cargo package"


def test_e2e_packaging_build_unredirected_when_the_package_root_is_the_repo_root(run_detect):
    out = run_detect(
        root_files={"Cargo.toml": '[package]\nname = "x"\n', "src/lib.rs": "pub fn f() {}\n"},
    )
    assert out["packaging_build"] == "cargo package"


def test_is_workspace_member_true_when_an_ancestor_up_to_repo_root_declares_a_workspace(tmp_path):
    repo_root = tmp_path
    (repo_root / "Cargo.toml").write_text('[workspace]\nmembers = ["packages/rust"]\n')
    package_root = repo_root / "packages" / "rust"
    package_root.mkdir(parents=True)
    assert detect.is_workspace_member(package_root, repo_root) is True


def test_is_workspace_member_false_when_no_ancestor_up_to_repo_root_declares_one(tmp_path):
    repo_root = tmp_path
    package_root = repo_root / "packages" / "rust"
    package_root.mkdir(parents=True)
    assert detect.is_workspace_member(package_root, repo_root) is False


def test_is_workspace_member_false_when_package_root_is_the_repo_root(tmp_path):
    assert detect.is_workspace_member(tmp_path, tmp_path) is False


def test_is_workspace_member_false_for_repo_root_package_even_with_an_outer_workspace(tmp_path_factory):
    base = tmp_path_factory.mktemp("outside-base")
    (base / "Cargo.toml").write_text('[workspace]\nmembers = ["repo"]\n')
    repo_root = base / "repo"
    repo_root.mkdir()
    assert detect.is_workspace_member(repo_root, repo_root) is False


def test_is_workspace_member_true_when_an_intermediate_ancestor_declares_a_workspace(tmp_path):
    repo_root = tmp_path
    mid = repo_root / "mid"
    mid.mkdir()
    (mid / "Cargo.toml").write_text('[workspace]\nmembers = ["packages/rust"]\n')
    package_root = mid / "packages" / "rust"
    package_root.mkdir(parents=True)
    assert detect.is_workspace_member(package_root, repo_root) is True


def test_is_workspace_member_falls_back_to_repo_root_when_package_root_is_unrelated(tmp_path_factory):
    package_root = tmp_path_factory.mktemp("package-tree")
    repo_root = tmp_path_factory.mktemp("repo-tree")
    (repo_root / "Cargo.toml").write_text('[workspace]\nmembers = ["x"]\n')
    assert detect.is_workspace_member(package_root, repo_root) is True


def test_is_workspace_member_never_searches_outside_repo_root(tmp_path_factory):
    base = tmp_path_factory.mktemp("outside-base")
    (base / "Cargo.toml").write_text('[workspace]\nmembers = ["repo/packages/rust"]\n')
    repo_root = base / "repo"
    package_root = repo_root / "packages" / "rust"
    package_root.mkdir(parents=True)
    assert detect.is_workspace_member(package_root, repo_root) is False


def test_e2e_cargo_target_dir_unredirected_for_a_standalone_crate_with_no_workspace(run_detect):
    out = run_detect(
        sources={"Cargo.toml": '[package]\nname = "x"\n', "src/lib.rs": "pub fn f() {}\n"},
    )
    assert out["cargo_target_dir"] == "scan/target"


def test_e2e_cargo_target_dir_workspace_member_redirects_to_the_workspace_root(run_detect):
    out = run_detect(
        scan_path="packages/rust/src",
        root_files={
            "Cargo.toml": '[workspace]\nmembers = ["packages/rust"]\n',
            "packages/rust/Cargo.toml": '[package]\nname = "x"\n',
            "packages/rust/src/lib.rs": "pub fn f() {}\n",
        },
    )
    assert out["cargo_target_dir"] == "./target"


def test_e2e_cargo_target_dir_unredirected_for_a_crate_that_is_itself_the_workspace_root(run_detect):
    out = run_detect(
        sources={
            "Cargo.toml": '[package]\nname = "x"\n\n[workspace]\nmembers = ["."]\n',
            "src/lib.rs": "pub fn f() {}\n",
        },
    )
    assert out["cargo_target_dir"] == "scan/target"


def test_e2e_cargo_target_dir_defaults_to_the_repo_root_target_with_no_rust(run_detect):
    out = run_detect()
    assert out["cargo_target_dir"] == "./target"


def test_e2e_packaging_build_prefers_the_wheel_for_a_pyo3_binding(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": '[project]\nname = "x"\n',
            "packages/py/Cargo.toml": '[package]\nname = "core"\n',
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["packaging_build"] == "uv build"
    assert out["packaging_language"] == "python"


def test_e2e_packaging_build_empty_when_the_manifest_cant_state_it(run_detect):
    out = run_detect(
        scan_path="packages/py/src",
        root_files={
            "packages/py/pyproject.toml": "[tool.black]\nline-length = 100\n",
            "packages/py/src/widget.py": "x = 1\n",
        },
    )
    assert out["packaging_build"] == ""
    assert out["packaging_language"] == ""


def test_e2e_packaging_build_empty_on_a_malformed_cargo(run_detect):
    out = run_detect(
        sources={"Cargo.toml": "not valid toml [[[", "src/lib.rs": "pub fn f() {}\n"},
    )
    assert out["packaging_build"] == ""


def test_e2e_packaging_build_empty_when_no_manifest_names_a_language(run_detect):
    out = run_detect(sources={"src/widget.py": "x = 1\n"})
    assert out["packaging_build"] == ""
    assert out["packaging_language"] == ""


def test_e2e_build_command_is_read_from_the_typescript_table(run_detect):
    out = run_detect(
        scan_path="packages/ts/src",
        root_files={
            "packages/ts/package.json": '{"name": "x"}\n',
            "packages/ts/testing-conventions.toml": '[typescript]\nbuild_command = "pnpm build"\n',
            "packages/ts/src/widget.ts": "export const x = 1;\n",
        },
    )
    assert out["build_command"] == "pnpm build"


def test_hermetic_outputs_for_this_repos_own_caller(run_detect):
    outputs = run_detect(caller_repository="thekevinscott/testing-conventions")
    assert outputs["cli_command"] == "./hermetic-cli/testing-conventions"
    assert (
        outputs["ts_mutation_adapter_args"]
        == "--ts-mutation-adapter ./hermetic-cli/dist/mutation/main.js"
    )


def test_published_outputs_for_another_caller(run_detect):
    outputs = run_detect(caller_repository="someone/else")
    assert outputs["cli_command"] == ""
    assert outputs["ts_mutation_adapter_args"] == ""


def test_published_outputs_when_a_version_is_pinned(run_detect):
    outputs = run_detect(
        caller_repository="thekevinscott/testing-conventions", version="0.3.0"
    )
    assert outputs["cli_command"] == ""


def test_every_emitted_output_is_declared_by_the_composite_action(run_detect):
    # An emitted output the manifest omits reaches the caller as the empty string, which an
    # expression with a `||` fallback takes as the fallback arm — green, with no signal.
    emitted = set(run_detect())
    assert emitted == set(_declared_outputs())


def test_every_declared_output_forwards_the_scan_step_of_the_same_name(run_detect):
    # The other half of that contract: a declaration wired to the wrong step output — a typo,
    # or a name left behind by a rename — forwards the empty string exactly like a missing one.
    declared = _declared_outputs()
    assert declared  # the block parsed; an empty dict would pass the loop vacuously
    for name, value in declared.items():
        assert value == "${{ steps.scan.outputs." + name + " }}"


def test_the_detect_job_forwards_exactly_the_outputs_the_action_declares():
    # A name the job's `outputs:` block omits reaches every rule job as the empty string, exactly
    # as a name the manifest omits does — `static_languages`' failure mode, one link downstream.
    assert set(_detect_job_outputs(WORKFLOW_YML.read_text())) == set(_declared_outputs())


def test_every_detect_job_output_forwards_both_scan_steps_of_the_same_name():
    # `uses:` cannot be dynamic, so each output reads the hermetic step or the published one. A
    # rename reaching one arm of the `||` and not the other still renders — as the empty string.
    outputs = _detect_job_outputs(WORKFLOW_YML.read_text())
    assert outputs  # the block parsed; an empty dict would pass the loop vacuously
    for name, value in outputs.items():
        hermetic = "${{ steps.scan_hermetic.outputs." + name
        assert value == hermetic + " || steps.scan_published.outputs." + name + " }}"


def test_every_needs_detect_reference_is_a_declared_job_output():
    # The last link: a rule job reading a name the `detect` job never declared gets the empty
    # string, and a declared output nobody reads is a derivation computed for no one.
    text = WORKFLOW_YML.read_text()
    assert _detect_references(text) == set(_detect_job_outputs(text))


def test_the_output_names_that_carry_a_digit_survive_the_parse():
    # The `[a-z_]+` trap: these three drop out of every set at once, so the comparisons above stay
    # true while covering two thirds of the chain.
    parsed = set(_detect_job_outputs(WORKFLOW_YML.read_text()))
    assert {"e2e_attestation", "e2e_extra_scope", "e2e_exclude"} <= parsed


def test_a_dropped_job_output_breaks_the_forwarding_set():
    # Non-vacuity, against the real file: drop one output the workflow still references.
    text = WORKFLOW_YML.read_text()
    dropped = re.sub(r"^      package_root: .*\n", "", text, count=1, flags=re.M)
    assert dropped != text
    assert "package_root" not in _detect_job_outputs(dropped)
    assert "package_root" in _detect_references(dropped)
    assert set(_detect_job_outputs(dropped)) != set(_declared_outputs())


def test_a_forward_wired_to_another_steps_output_breaks_the_same_name_rule():
    text = WORKFLOW_YML.read_text().replace(
        "steps.scan_published.outputs.package_root", "steps.scan_published.outputs.packaging_dist", 1
    )
    outputs = _detect_job_outputs(text)
    assert "packaging_dist" in outputs["package_root"]
    assert not outputs["package_root"].endswith("scan_published.outputs.package_root }}")
