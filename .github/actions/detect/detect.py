#!/usr/bin/env python3
"""Detect which languages (and conditions) are present, for the reusable workflow.

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it is invoked by the reusable workflow's `detect` job to decide
which per-language rule jobs to fan out over. Keeping the logic here — rather than inline in
the workflow YAML — lets it carry real integration and e2e tests (see `tests/`).

The filesystem is the one external dependency. It lives behind the small boundary functions
below (`has_source`, `has_rust_crate`, `has_dist`, …) so an *integration* test can mock it and
exercise the real `compute_outputs` orchestration, while an *e2e* test runs the whole thing
against a real directory tree.

Inputs come from the environment (set by the workflow):
  LANGUAGES      JSON array restricting the languages to check; empty restricts nothing.
  SCAN_PATH      directory scanned for sources (default '.').
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


_DIST_GLOBS: tuple[str, ...] = ("*.whl", "*.tar.gz", "*.tgz", "*.crate")


def has_dist(root: Path) -> bool:
    """True if a conventional `dist/` under `root` holds a recognized built distribution.

    Lets packaging run by default (#186): the rule inspects whatever dist is discoverable.
    """
    dist = root / "dist"
    return dist.is_dir() and _any_match(dist, _DIST_GLOBS)


def has_attestation(root: Path) -> bool:
    """True if a committed `e2e-attestation.json` sits at `root` (the e2e-verify default-on, #186)."""
    return (root / "e2e-attestation.json").is_file()


# --- orchestration (runs for real under both test kinds; only the fs is mocked) ---

def eligible(languages_input: str, language: str) -> bool:
    """Whether `language` is in scope, given the raw `LANGUAGES` restrictor.

    `languages_input` is an optional restrictor: empty (or `[]`) puts every supported language
    in scope — auto-detect (#185); a non-empty JSON array restricts to the languages it names.
    """
    restrictor = languages_input.strip()
    return restrictor in ("", "[]") or f'"{language}"' in restrictor


def _to_json(languages: list[str]) -> str:
    """Compact JSON array, matching what the matrix `fromJSON(...)` consumes (`[]` when empty)."""
    return json.dumps(languages, separators=(",", ":"))


def compute_outputs(
    languages_input: str, scan_root: str, repo_root: str = "."
) -> dict[str, str]:
    """The detected sets, as `name -> value` strings for GITHUB_OUTPUT.

    `languages` (python/typescript present) drives colocated-test + coverage; the lint/isolation
    and coverage sets add Rust whenever a crate is present — Rust coverage is zero-config now
    (`lines = 100` by default, #206), so it no longer waits for a configured floor.
    `packaging_dist` / `e2e_attestation` (looked for at `repo_root`, the checkout root) let the
    packaging and e2e-verify jobs run by default and skip — never fail — when absent (#186).
    """
    root = Path(scan_root)
    present = [
        language
        for language in ("python", "typescript")
        if eligible(languages_input, language) and has_source(root, language)
    ]
    rust_crate = eligible(languages_input, "rust") and has_rust_crate(root)
    with_rust = present + (["rust"] if rust_crate else [])
    repo = Path(repo_root)
    return {
        "languages": _to_json(present),
        "integration_lint_languages": _to_json(with_rust),
        "isolation_languages": _to_json(with_rust),
        "coverage_languages": _to_json(with_rust),
        "packaging_dist": "true" if has_dist(repo) else "false",
        "e2e_attestation": "true" if has_attestation(repo) else "false",
    }


def main() -> int:
    languages = os.environ.get("LANGUAGES", "")
    scan_path = os.environ.get("SCAN_PATH", ".")
    outputs = compute_outputs(languages, scan_path)

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
