"""End-to-end tests for the detect script.

Per the standard, an e2e test runs with no mocks. The real filesystem is the input: each test
builds a temporary directory tree and runs `detect.py` as a subprocess exactly as the workflow
does, then reads back the `name=value` lines it wrote to GITHUB_OUTPUT.
"""
import os
import subprocess
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "detect.py"


def _run(tmp_path, *, languages="", scan="."):
    out_file = tmp_path / "github_output"
    out_file.write_text("")
    env = {
        **os.environ,
        "LANGUAGES": languages,
        "SCAN_PATH": scan,
        "GITHUB_OUTPUT": str(out_file),
    }
    subprocess.run([sys.executable, str(SCRIPT)], cwd=tmp_path, env=env, check=True)
    return dict(
        line.split("=", 1) for line in out_file.read_text().splitlines() if "=" in line
    )


def _mk(tmp_path, rel, content="x"):
    path = tmp_path / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


# --- baseline: current behavior (green) ---

def test_e2e_explicit_python(tmp_path):
    _mk(tmp_path, "src/widget.py", "x = 1\n")
    out = _run(tmp_path, languages='["python"]', scan="src")
    assert out["languages"] == '["python"]'
    assert out["coverage_languages"] == '["python"]'


def test_e2e_explicit_rust_routes_into_coverage_zero_config(tmp_path):
    # Rust coverage is zero-config now (#206): a crate enters the coverage matrix with
    # no `[rust].coverage` floor configured — the default `lines = 100` applies.
    _mk(tmp_path, "Cargo.toml", '[package]\nname = "x"\n')
    _mk(tmp_path, "src/lib.rs", "pub fn f() {}\n")
    out = _run(tmp_path, languages='["rust"]', scan=".")
    assert out["integration_lint_languages"] == '["rust"]'
    assert out["coverage_languages"] == '["rust"]'


def test_e2e_absent_language_skipped(tmp_path):
    _mk(tmp_path, "src/widget.py", "x = 1\n")
    out = _run(tmp_path, languages='["python","typescript"]', scan="src")
    assert out["languages"] == '["python"]'


# --- #185 auto-detect (RED until implemented) ---

def test_e2e_empty_languages_auto_detects_rust(tmp_path):
    _mk(tmp_path, "Cargo.toml", '[package]\nname = "x"\n')
    _mk(tmp_path, "src/lib.rs", "pub fn f() {}\n")
    out = _run(tmp_path, languages="", scan=".")
    assert '"rust"' in out["integration_lint_languages"]


def test_e2e_empty_languages_auto_detects_python(tmp_path):
    _mk(tmp_path, "src/widget.py", "x = 1\n")
    out = _run(tmp_path, languages="", scan="src")
    assert out["languages"] == '["python"]'


# --- #186 packaging_dist / e2e_attestation (RED until implemented) ---

def test_e2e_packaging_dist_located(tmp_path):
    _mk(tmp_path, "dist/widget-0.1.0-py3-none-any.whl", "")
    out = _run(tmp_path, languages="", scan=".")
    assert out["packaging_dist"] == "true"


def test_e2e_attestation_detected(tmp_path):
    _mk(tmp_path, "e2e-attestation.json", "{}")
    out = _run(tmp_path, languages="", scan=".")
    assert out["e2e_attestation"] == "true"
