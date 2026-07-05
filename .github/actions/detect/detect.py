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
  CONFIG         the config file `--config` should receive (default 'testing-conventions.toml');
                 the derived value is the caller's value verbatim unless it equals the default,
                 in which case a package-root testing-conventions.toml wins if present (#277).
  GITHUB_OUTPUT  file the detected sets are appended to (`name=value` lines).
"""
from __future__ import annotations

import json
import os
import tomllib
from pathlib import Path

# Source globs per language. Rust is a crate (a Cargo.toml or any *.rs), detected separately.
_SOURCE_GLOBS: dict[str, tuple[str, ...]] = {
    "python": ("*.py",),
    "typescript": ("*.ts", "*.tsx", "*.mts", "*.cts"),
}


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
    """True if `root` holds a Rust crate *to check* — a `Cargo.toml` **and** at least one
    `.rs` source. A manifest alone has nothing to measure: e.g. `packages/python` carries a
    `Cargo.toml` but generates its Rust sources at wheel-build time, so a plain checkout has
    no `.rs`. Treating that as a crate makes the zero-config rust coverage/mutation jobs
    (#206) run `cargo` over absent sources and fail; requiring real source skips it instead.
    """
    return _any_match(root, ("Cargo.toml",)) and _any_match(root, ("*.rs",))


_DIST_GLOBS: tuple[str, ...] = ("*.whl", "*.tar.gz", "*.tgz", "*.crate")


def has_dist(root: Path) -> bool:
    """True if a conventional `dist/` under `root` holds a recognized built distribution.

    Lets packaging run by default (#186): the rule inspects whatever dist is discoverable.
    """
    dist = root / "dist"
    return dist.is_dir() and _any_match(dist, _DIST_GLOBS)


def has_attestation(root: Path) -> bool:
    """True if a committed `e2e-attestation.json` sits at `root` (the e2e-verify default-on,
    #186). Called with the package root, not the checkout root (#281): a monorepo package
    carries its own attestation, scoped to it exactly like `e2e verify <path>` is.
    """
    return (root / "e2e-attestation.json").is_file()


_MANIFESTS: tuple[str, ...] = ("package.json", "pyproject.toml", "Cargo.toml")


def has_manifest(root: Path) -> bool:
    """True if a package manifest (package.json / pyproject.toml / Cargo.toml) sits directly at
    `root` (the monorepo package-root primitive, #277).
    """
    return any((root / name).is_file() for name in _MANIFESTS)


def read_package_json(root: Path) -> dict:
    """The parsed `package.json` at `root`, or `{}` if absent or unparseable."""
    manifest = root / "package.json"
    if not manifest.is_file():
        return {}
    try:
        return json.loads(manifest.read_text())
    except (OSError, json.JSONDecodeError):
        return {}


def read_pyproject(root: Path) -> dict:
    """The parsed `pyproject.toml` at `root`, or `{}` if absent or unparseable."""
    manifest = root / "pyproject.toml"
    if not manifest.is_file():
        return {}
    try:
        return tomllib.loads(manifest.read_text())
    except tomllib.TOMLDecodeError:
        return {}


def has_lockfile(root: Path, name: str) -> bool:
    """True if a file named `name` sits directly at `root`."""
    return (root / name).is_file()


# --- orchestration (runs for real under both test kinds; only the fs is mocked) ---


def derive_package_root(scan_root: Path, repo_root: Path) -> Path:
    """The package root: the nearest directory at-or-above `scan_root`, down to `repo_root`
    inclusive, holding a manifest; `repo_root` when none is found (#277). A single-package repo
    (no manifest above `scan_root` other than possibly at the checkout root) always derives
    `repo_root`, so every existing consumer is untouched.

    `working_directory` was considered and rejected as a second, consumer-facing coordinate
    system (docs/monorepo.md): `path` stays the only scoping input, and everything else the
    suite-executing jobs need is derived from it and the package's own manifest.
    """
    scan_root = scan_root.resolve()
    repo_root = repo_root.resolve()
    candidates = [scan_root]
    current = scan_root
    while current != repo_root and current.parent != current:
        current = current.parent
        candidates.append(current)
    if repo_root not in candidates:
        candidates.append(repo_root)
    for candidate in candidates:
        if has_manifest(candidate):
            return candidate
    return repo_root


def _package_manager_from_field(value: str) -> str | None:
    """The manager name from a `package.json` `packageManager` value like `pnpm@8.6.0`, or
    `None` when the field is empty.
    """
    return value.split("@", 1)[0] if value else None


def ts_package_manager(package_root: Path) -> str:
    """The TypeScript package manager `package_root` is set up for (#277): the name declared in
    `package.json`'s `packageManager` field, else `pnpm` when a `pnpm-lock.yaml` sits alongside
    it, else `npm` when a `package-lock.json` does, else `pnpm` (today's hardcoded default, so an
    unrecognized single-package repo is unchanged).
    """
    declared = _package_manager_from_field(read_package_json(package_root).get("packageManager", ""))
    if declared:
        return declared
    if has_lockfile(package_root, "pnpm-lock.yaml"):
        return "pnpm"
    if has_lockfile(package_root, "package-lock.json"):
        return "npm"
    return "pnpm"


def python_env(package_root: Path) -> str:
    """The Python environment model `package_root` is set up for (#277): `uv` when its
    `pyproject.toml` declares a `[project]` table (an installable project with its own
    dependencies), else `pip` — no pyproject.toml at all, one with only tool config, or one that
    fails to parse (detect never crashes on a malformed manifest).
    """
    return "uv" if "project" in read_pyproject(package_root) else "pip"


def provision_rust(package_root: Path) -> str:
    """`"true"` when `package_root`'s own manifest declares a Rust-compiling build, so the suite
    jobs can provision cargo with no `rust_toolchain` input (#277): a `Cargo.toml` sits there; or
    `pyproject.toml`'s `build-system.build-backend` is a maturin backend; or `package.json`
    declares a `napi` key or an `@napi-rs/cli` devDependency. `"false"` otherwise — `rust_toolchain`
    remains a manual override for a build no manifest field expresses.
    """
    if has_lockfile(package_root, "Cargo.toml"):
        return "true"
    backend = read_pyproject(package_root).get("build-system", {}).get("build-backend", "")
    if backend.startswith("maturin"):
        return "true"
    package = read_package_json(package_root)
    if "napi" in package:
        return "true"
    if "@napi-rs/cli" in package.get("devDependencies", {}):
        return "true"
    return "false"


_CONFIG_DEFAULT = "testing-conventions.toml"


def derive_config(package_root_rel: Path, config_input: str) -> str:
    """The config file `--config` should receive (#277): `config_input` verbatim when the
    caller named anything other than the default; otherwise `testing-conventions.toml` at
    `package_root_rel` when that file exists there, else the default itself — today's
    repo-root behavior, unchanged when `package_root_rel` is `.`. An explicit override always
    wins, so a caller wanting the repo-root file from a scoped call passes an unambiguous path
    (e.g. `./testing-conventions.toml`).
    """
    if config_input != _CONFIG_DEFAULT:
        return config_input
    candidate = package_root_rel / _CONFIG_DEFAULT
    return str(candidate) if candidate.is_file() else _CONFIG_DEFAULT

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
    languages_input: str, scan_root: str, repo_root: str = ".", config_input: str = _CONFIG_DEFAULT
) -> dict[str, str]:
    """The detected sets, as `name -> value` strings for GITHUB_OUTPUT.

    `languages` (python/typescript present) drives the co-change job — rust units are inline
    `#[cfg(test)]` modules, so a sibling test can't go stale and co-change doesn't apply. The
    whole-tree colocated-test set and the lint/isolation and coverage sets add Rust whenever a
    crate is present — the rust presence arm checks the inline module (#40/#274), and Rust
    coverage is zero-config now (`lines = 100` by default, #206), so neither waits for config.
    `packaging_dist` (looked for at `repo_root`, the checkout root) and `e2e_attestation`
    (looked for at `package_root`, #281 — a monorepo package's own attestation, not the
    checkout root) let the packaging and e2e-verify jobs run by default and skip — never
    fail — when absent (#186).
    `package_root` / `ts_package_manager` / `python_env` / `provision_rust` / `config` (#277)
    are the monorepo primitive: everything a suite-executing job needs to install, build, run,
    and configure at the right directory, derived from `scan_root` and the nearest manifest
    rather than a second, consumer-facing scoping input.
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
    package_root = derive_package_root(root, repo)
    try:
        package_root_rel = package_root.relative_to(repo.resolve())
    except ValueError:
        package_root_rel = Path(".")
    return {
        "languages": _to_json(present),
        # Whole-tree colocated-test (#274): the file-paired languages plus rust — the rust
        # arm checks inline `#[cfg(test)]` presence (#40), so a crate rides the matrix too.
        "colocated_test_languages": _to_json(with_rust),
        "integration_lint_languages": _to_json(with_rust),
        "isolation_languages": _to_json(with_rust),
        "coverage_languages": _to_json(with_rust),
        # `unit mutation` (#204): the same set as coverage — present python/typescript plus
        # rust when a crate is here — now that all three arms are at parity (#201/#202/#203).
        "mutation_languages": _to_json(with_rust),
        "packaging_dist": "true" if has_dist(repo) else "false",
        # #281: scoped to the package root, not the checkout root — a monorepo package
        # carries its own attestation, exactly like `e2e verify <path>` checks it.
        "e2e_attestation": "true" if has_attestation(package_root) else "false",
        "package_root": str(package_root_rel),
        "ts_package_manager": ts_package_manager(package_root),
        "python_env": python_env(package_root),
        "provision_rust": provision_rust(package_root),
        "config": derive_config(package_root_rel, config_input),
    }


def main() -> int:
    languages = os.environ.get("LANGUAGES", "")
    scan_path = os.environ.get("SCAN_PATH", ".")
    config_input = os.environ.get("CONFIG", _CONFIG_DEFAULT)
    outputs = compute_outputs(languages, scan_path, config_input=config_input)

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
