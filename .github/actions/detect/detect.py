#!/usr/bin/env python3
"""Detect which languages (and conditions) are present, for the reusable workflow.

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it is invoked by the reusable workflow's `detect` job to decide
which per-language rule jobs to fan out over. Keeping the logic here — rather than inline in
the workflow YAML — lets it carry real integration and e2e tests (see `tests/`).

The filesystem is the one external dependency. It lives behind the small boundary functions
below (`has_source`, `has_rust_crate`, `rust_coverage_configured`, …) so an *integration* test
can mock it and exercise the real `compute_outputs` orchestration, while an *e2e* test runs the
whole thing against a real directory tree.

Inputs come from the environment (set by the workflow):
  LANGUAGES      JSON array restricting the languages to check; empty restricts nothing.
  SCAN_PATH      directory scanned for sources (default '.').
  CONFIG         path to the config file (for the [rust].coverage floor).
  GITHUB_OUTPUT  file the detected sets are appended to (`name=value` lines).
"""
from __future__ import annotations

import json
import os
from pathlib import Path

# Source globs per language. Rust is a crate (a Cargo.toml or any *.rs), detected separately.
_SOURCE_GLOBS: dict[str, tuple[str, ...]] = {
    "python": ("*.py",),
    "typescript": ("*.ts", "*.tsx", "*.mts", "*.cts"),
}
_RUST_GLOBS: tuple[str, ...] = ("Cargo.toml", "*.rs")


# --- filesystem boundary (the external dependency an integration test mocks) ---

def _any_match(root: Path, globs: tuple[str, ...]) -> bool:
    """True if any file matching one of `globs` exists anywhere under `root`."""
    for glob in globs:
        for _ in root.rglob(glob):
            return True
    return False


def has_source(root: Path, language: str) -> bool:
    """True if `root` holds any source file for `language` (python / typescript)."""
    return _any_match(root, _SOURCE_GLOBS[language])


def has_rust_crate(root: Path) -> bool:
    """True if `root` holds a Rust crate — a Cargo.toml or any *.rs."""
    return _any_match(root, _RUST_GLOBS)


def rust_coverage_configured(config_path: str) -> bool:
    """True if the config at `config_path` declares a `[rust].coverage` floor.

    Rust has no default coverage floor, so its coverage arm only runs when one is configured.
    """
    if not config_path or not Path(config_path).is_file():
        return False
    text = Path(config_path).read_text()
    try:
        import tomllib

        return "coverage" in tomllib.loads(text).get("rust", {})
    except Exception:
        import re

        if re.search(r"(?m)^\s*\[rust\.coverage\]", text):
            return True
        table = re.search(r"(?ms)^\[rust\][^\[]*", text)
        return bool(table and re.search(r"(?m)^\s*coverage\s*=", table.group(0)))


# --- orchestration (runs for real under both test kinds; only the fs is mocked) ---

def eligible(languages_input: str, language: str) -> bool:
    """Whether `language` is in scope, given the raw `LANGUAGES` restrictor.

    Today: a language is eligible only when explicitly named in `languages_input`.
    """
    return f'"{language}"' in languages_input


def _to_json(languages: list[str]) -> str:
    """Compact JSON array, matching what the matrix `fromJSON(...)` consumes (`[]` when empty)."""
    return json.dumps(languages, separators=(",", ":"))


def compute_outputs(languages_input: str, scan_root: str, config_path: str) -> dict[str, str]:
    """The detected sets, as `name -> value` strings for GITHUB_OUTPUT.

    `languages` (python/typescript present) drives colocated-test + coverage; the lint/isolation
    sets add Rust when a crate is present; coverage adds Rust only when a floor is configured.
    """
    root = Path(scan_root)
    present = [
        language
        for language in ("python", "typescript")
        if eligible(languages_input, language) and has_source(root, language)
    ]
    rust_crate = eligible(languages_input, "rust") and has_rust_crate(root)
    with_rust = present + (["rust"] if rust_crate else [])
    coverage = present + (["rust"] if rust_crate and rust_coverage_configured(config_path) else [])
    return {
        "languages": _to_json(present),
        "integration_lint_languages": _to_json(with_rust),
        "isolation_languages": _to_json(with_rust),
        "coverage_languages": _to_json(coverage),
    }


def main() -> int:
    languages = os.environ.get("LANGUAGES", "")
    scan_path = os.environ.get("SCAN_PATH", ".")
    config = os.environ.get("CONFIG", "")
    outputs = compute_outputs(languages, scan_path, config)

    github_output = os.environ.get("GITHUB_OUTPUT")
    if github_output:
        with open(github_output, "a", encoding="utf-8") as handle:
            for name, value in outputs.items():
                handle.write(f"{name}={value}\n")
    summary = ", ".join(f"{name} {value}" for name, value in outputs.items())
    print(f"languages='{languages}' under '{scan_path}' -> {summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
