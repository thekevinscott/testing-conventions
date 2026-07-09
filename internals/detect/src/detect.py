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
  CALLER_REPOSITORY  the calling run's `github.repository` (#356) — for a reusable workflow this
                 always belongs to the *caller*, so hermetic mode is derivable only for this
                 repo's own workflows; empty (the default) means the published path.
  VERSION        the workflow's `version` input (#356): an explicit published version always wins
                 over hermetic mode (the #357 post-publish verification path).
  GITHUB_OUTPUT  file the detected sets are appended to (`name=value` lines).
"""
from __future__ import annotations

import hashlib
import json
import os
import tomllib
from pathlib import Path
from typing import Optional

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
    candidates = []
    for ancestor in [scan_root, *scan_root.parents]:
        candidates.append(ancestor)
        if ancestor == repo_root:
            break
    else:
        # The walk never reached repo_root (scan_root isn't under it): fall back to checking
        # repo_root itself, without crossing its boundary outward.
        candidates.append(repo_root)
    for candidate in candidates:
        if has_manifest(candidate):
            return candidate
    return repo_root


def _package_manager_from_field(value: str) -> Optional[str]:
    """The manager name from a `package.json` `packageManager` value like `pnpm@8.6.0`, or
    `None` when the field is empty.
    """
    return value.partition("@")[0] if value else None


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


def build_command_language(primary: str, present: list[str]) -> str:
    """The language `derive_build_command` should read its `[<language>].build_command` table
    from: `primary` (the manifest-derived `primary_language`) when there is one, else the single
    entry in `present` (the file-paired languages `has_source` found) when that's unambiguous —
    a manifest-less pip Python package (#289's original case) has no `primary_language` to key
    on, but its language is unambiguous when exactly one of python/typescript is present. `''`
    (no fallback, never a guess) when `present` holds zero or more than one language and there is
    no manifest to disambiguate (#354/#355). Unpacking (not a `len(...) == 1` comparison) names
    the "exactly one" boundary without a numeric literal to guess at.
    """
    if primary:
        return primary
    try:
        (sole,) = present
    except ValueError:
        return ""
    return sole


def derive_build_command(config: str, language: str) -> str:
    """The `[<language>].build_command` build declaration (#289, generalized to all languages in
    #335) read from the in-effect config file `config` (the path `derive_config` resolved), or
    `''` when that file is absent/unparseable, `language` is empty, or the table declares no
    `build_command`.

    It lives in the package's own `testing-conventions.toml` rather than on the `uses:` call —
    discovered exactly like `config` itself — and names a build the manifest structurally can't
    express: a PEP 517 Python backend's missing pre-build shell step, or a TypeScript
    compile-before-`pack` in a `build` script npm doesn't standardize the name of. `language` is
    normally the package's primary language (`primary_language`), so a package's own table is
    read — but callers pass the single present language as a fallback when there is no manifest
    to derive a primary from (a bare pip Python package, #289's original case has none), since
    `primary_language` alone would otherwise silently drop the build step for every manifest-less
    package (#355). `''` means no build step. Parsed with the same stdlib `tomllib`, so a
    malformed config never crashes detect.
    """
    if not language:
        return ""
    path = Path(config)
    if not path.is_file():
        return ""
    try:
        data = tomllib.loads(path.read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return ""
    value = data.get(language, {}).get("build_command", "")
    return value if isinstance(value, str) else ""


def primary_language(package_root: Path) -> str:
    """The package's primary language by manifest, or `''` when none is present (#335): a
    `pyproject.toml` is `python`, else a `package.json` is `typescript`, else a `Cargo.toml` is
    `rust`. The priority resolves a binding, which carries two manifests: a PyO3 binding
    (`pyproject.toml` + `Cargo.toml`) publishes a Python wheel, a napi binding (`package.json` +
    `Cargo.toml`) publishes an npm tarball — the second manifest is the private core, not the
    published artifact. Drives which `[<language>]` table `build_command` is read from, and which
    build `derive_packaging` picks.
    """
    if (package_root / "pyproject.toml").is_file():
        return "python"
    if (package_root / "package.json").is_file():
        return "typescript"
    if (package_root / "Cargo.toml").is_file():
        return "rust"
    return ""


def read_cargo(package_root: Path) -> dict:
    """The parsed `Cargo.toml` at `package_root`, or `{}` if absent or unparseable. Only called
    once `primary_language` has seen a `Cargo.toml`, so a missing file is handled by the same
    `OSError` catch as an unreadable one — no separate presence guard."""
    try:
        return tomllib.loads((package_root / "Cargo.toml").read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return {}


def is_workspace_member(package_root: Path, repo_root: Path) -> bool:
    """True when `package_root`'s crate belongs to a Cargo workspace rooted at an ancestor (#360):
    some directory strictly above `package_root`, down to `repo_root` inclusive, has a
    `Cargo.toml` with a `[workspace]` table. Cargo resolves the target directory — and so `cargo
    package`'s output — at the *workspace* root regardless of the invoking working directory, so
    a workspace member's derived build must redirect `--target-dir` back to its own tree rather
    than let the crate land where the packaging job's scan never looks.

    A crate whose own `Cargo.toml` carries both `[package]` and `[workspace]` (a workspace-root
    package) is not a *member* of an ancestor workspace — its own target dir is already correct,
    so this only inspects ancestors, never `package_root` itself.
    """
    package_root = package_root.resolve()
    repo_root = repo_root.resolve()
    if package_root == repo_root:
        return False
    ancestors = []
    for ancestor in package_root.parents:
        ancestors.append(ancestor)
        if ancestor == repo_root:
            break
    else:
        # The walk never reached repo_root (package_root isn't under it): fall back to
        # checking repo_root itself, mirroring `derive_package_root`'s own boundary handling.
        ancestors.append(repo_root)
    return any("workspace" in read_cargo(ancestor) for ancestor in ancestors)


def derive_packaging(package_root: Path, primary: str, repo_root: Path) -> str:
    """The command that builds the package's publishable distribution from its manifest alone
    (#335), or `''` when the manifest doesn't standardize one. The reusable packaging job runs it
    at the package root, then scans the built artifact — so a native-building monorepo adopts the
    gate with `gates: ["packaging"]` and no bespoke build job:

    - **Python** (a `pyproject.toml` with a `[project]` table) → `uv build` — the PEP 517 build,
      which writes `dist/*.whl` + `*.tar.gz` and compiles a maturin/PyO3 core along the way.
    - **TypeScript** (a `package.json`) → `<pm> pack --pack-destination dist` — a standard `npm` /
      `pnpm` command that runs the package's own `prepare` / `prepack` lifecycle. A compile that
      lives in a bare `build` script instead (whose name npm doesn't standardize) is named in
      `[typescript].build_command`, run first.
    - **Rust** (a `Cargo.toml` with a `[package]` table) → `cargo package` — writes
      `target/package/*.crate`. When `package_root` is a member of an ancestor Cargo workspace
      (#360), that command redirects with `--target-dir target` so the crate lands at the
      package's own `target/package/` instead of the workspace root's.

    `''` for a package whose manifest can't state the build (a non-`[project]` pyproject, a
    workspace-only `Cargo.toml`): the job then scans a committed `dist/` if present, or a
    `packaging_artifact`, exactly as before.

    Dispatched on `primary` through a table rather than a chain of `primary == "…"` branches, so
    which language builds is an exact key lookup — a language the table doesn't name yields no
    build, no comparison to misjudge.
    """
    def rust_package() -> str:
        if "package" not in read_cargo(package_root):
            return ""
        if is_workspace_member(package_root, repo_root):
            return "cargo package --target-dir target"
        return "cargo package"

    builders = {
        "python": lambda: "uv build" if "project" in read_pyproject(package_root) else "",
        "typescript": lambda: f"{ts_package_manager(package_root)} pack --pack-destination dist",
        "rust": rust_package,
    }
    build = builders.get(primary)
    return build() if build else ""


def derive_e2e_extra_scope(config: str) -> str:
    """The `[e2e].extra_scope` freshness roots (#333) read from the in-effect config file
    `config`, rendered as repeated `--extra-scope <dir>` arguments for the e2e-verify run step,
    or `''` when that file is absent, unparseable, declares no `[e2e].extra_scope`, or gives a
    non-list. A package whose e2e artifact is compiled from a shared source tree beside it (a
    native core bound into several bindings) declares that tree here — discovered exactly like
    `config` and `build_command`, not passed on the `uses:` call.
    """
    return _e2e_scope_flags(config, "extra_scope", "--extra-scope")


def derive_e2e_exclude(config: str) -> str:
    """The `[e2e].exclude` feature-gated subtrees (#333) read from the in-effect config file
    `config`, rendered as repeated `--exclude <dir>` arguments, or `''` when none are declared —
    the carve-out for a core `cli/` compiled out of the bindings, so a change only under it stays
    fresh. Same discovery and shape as [`derive_e2e_extra_scope`].
    """
    return _e2e_scope_flags(config, "exclude", "--exclude")


def _e2e_scope_flags(config: str, key: str, flag: str) -> str:
    """The `[e2e].<key>` list from `config`, rendered as repeated `<flag> <dir>` arguments the
    e2e-verify run step appends verbatim, or `''` when the file is absent/unparseable or the key
    is missing or not a list of directory strings. Parsed with the same stdlib `tomllib` used for
    `pyproject.toml`, so a malformed config never crashes detect. Repo-relative directory paths
    only — they are word-split by the run step, so a path with a space would not survive.
    """
    path = Path(config)
    if not path.is_file():
        return ""
    try:
        data = tomllib.loads(path.read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return ""
    value = data.get("e2e", {}).get(key, [])
    if not isinstance(value, list):
        return ""
    return " ".join(f"{flag} {directory}" for directory in value if isinstance(directory, str) and directory)


def eligible(languages_input: str, language: str) -> bool:
    """Whether `language` is in scope, given the raw `LANGUAGES` restrictor.

    `languages_input` is an optional restrictor: empty (or `[]`) puts every supported language
    in scope — auto-detect (#185); a non-empty JSON array restricts to the languages it names.
    """
    restrictor = languages_input.strip()
    return restrictor in ("", "[]") or f'"{language}"' in restrictor


_HERMETIC_CALLER = "thekevinscott/testing-conventions"
_HERMETIC_CLI_COMMAND = "./hermetic-cli/testing-conventions"
_HERMETIC_TS_ADAPTER_ARGS = "--ts-mutation-adapter ./hermetic-cli/dist/mutation/main.js"


def hermetic(caller_repository: str, version: str) -> bool:
    """Whether this run gates the commit under test hermetically (#356).

    True iff the calling run is this repository's own (`caller_repository` is the reusable
    workflow's `github.repository`, which for a reusable workflow always belongs to the
    *caller*) and no explicit `version` was requested — an explicit version names the published
    artifact (the #357 post-publish verification path) and always wins. Every other caller takes
    the published `npx` path, so a consumer can neither trigger nor observe hermetic mode.
    """
    return caller_repository == _HERMETIC_CALLER and version == ""


def _to_json(languages: list[str]) -> str:
    """Compact JSON array, matching what the matrix `fromJSON(...)` consumes (`[]` when empty)."""
    return json.dumps(languages, separators=(",", ":"))


def compute_outputs(
    languages_input: str,
    scan_root: str,
    repo_root: str = ".",
    config_input: str = _CONFIG_DEFAULT,
    caller_repository: str = "",
    version: str = "",
) -> dict[str, str]:
    """The detected sets, as `name -> value` strings for GITHUB_OUTPUT.

    `languages` (python/typescript present) drives the co-change job — rust units are inline
    `#[cfg(test)]` modules, so a sibling test can't go stale and co-change doesn't apply. The
    whole-tree colocated-test set, the `static_languages` set (the `static` job's matrix, whose
    steps run the four static gates, #410), and the lint/isolation and coverage sets add Rust
    whenever a crate is present — the rust presence arm checks the inline module (#40/#274), and
    Rust coverage is zero-config now (`lines = 100` by default, #206), so neither waits for config.
    `packaging_dist` is looked for at the derived `package_root` (#280) — a per-package `uses:`
    call inspects only its own package's `dist/`; a repo-root `dist/` counts only when the
    derived package root IS the repo root, which every current single-package consumer's is.
    `e2e_attestation` is likewise looked for at `package_root` (#281) — a monorepo package
    carries its own attestation, exactly like `e2e verify <path>` checks it. Either lets the
    packaging and e2e-verify jobs run by default and skip — never fail — when absent (#186).
    `package_root` / `ts_package_manager` / `python_env` / `provision_rust` / `config` (#277)
    are the monorepo primitive: everything a suite-executing job needs to install, build, run,
    and configure at the right directory, derived from `scan_root` and the nearest manifest
    rather than a second, consumer-facing scoping input. `build_command` (#289) is the
    `[python].build_command` escape hatch, read from that same discovered `config` file rather
    than passed on the `uses:` call — the suite-executing jobs run it before the suite.
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
    config = derive_config(package_root_rel, config_input)
    primary = primary_language(package_root)
    bc_language = build_command_language(primary, present)
    packaging_build = derive_packaging(package_root, primary, repo)
    return {
        "languages": _to_json(present),
        # Whole-tree colocated-test (#274): the file-paired languages plus rust — the rust
        # arm checks inline `#[cfg(test)]` presence (#40), so a crate rides the matrix too.
        "colocated_test_languages": _to_json(with_rust),
        "integration_lint_languages": _to_json(with_rust),
        "isolation_languages": _to_json(with_rust),
        # The `static` job's matrix (#410): the four static gates — colocated-test (and its
        # co-change variant), unit-lint, integration-lint — run as its steps, fanned out over this
        # rust-inclusive union. The same set the colocated/isolation/integration sets already hold,
        # under its own name so a future language can diverge per set without breaking the matrix.
        "static_languages": _to_json(with_rust),
        "coverage_languages": _to_json(with_rust),
        # `unit mutation` (#204): the same set as coverage — present python/typescript plus
        # rust when a crate is here — now that all three arms are at parity (#201/#202/#203).
        "mutation_languages": _to_json(with_rust),
        "packaging_dist": "true" if has_dist(package_root) else "false",
        # #281: scoped to the package root, not the checkout root — a monorepo package
        # carries its own attestation, exactly like `e2e verify <path>` checks it.
        "e2e_attestation": "true" if has_attestation(package_root) else "false",
        "package_root": str(package_root_rel),
        "ts_package_manager": ts_package_manager(package_root),
        "python_env": python_env(package_root),
        "provision_rust": provision_rust(package_root),
        "config": config,
        # #289/#335: the `[<primary>].build_command` declaration, read from the package's own
        # config (`config` above) — the suite-executing and packaging jobs run it. Read from the
        # package's primary-language table, generalized from the old `[python]`-only lookup.
        "build_command": derive_build_command(config, bc_language),
        # #335: the standard artifact build derived from the manifest (`uv build` / `<pm> pack` /
        # `cargo package`), and the language to provision for it — so the packaging job builds the
        # distribution before scanning, no caller build job. Empty when the manifest can't state a
        # build (the job then scans a committed dist/ or a packaging_artifact, as before).
        "packaging_build": packaging_build,
        "packaging_language": primary if packaging_build else "",
        # #333: extra e2e freshness roots and their feature-gated excludes, read from the same
        # discovered `config` — a shared source tree beside the package that no `--scope`
        # at-or-below the package root can reach. Rendered as repeated `--extra-scope`/`--exclude`
        # arguments the e2e-verify run step appends; empty when the package declares none.
        "e2e_extra_scope": derive_e2e_extra_scope(config),
        "e2e_exclude": derive_e2e_exclude(config),
        # #356: the hermetic CLI invocation — the path where the caller workflows' build-cli job
        # staged this commit's own binary — when this repo gates itself with no pinned version;
        # empty (-> the run line's npx fallback) for every other caller. The TS mutation adapter
        # argument rides along pre-rendered (the #333 e2e_extra_scope pattern): the npm launcher
        # normally appends it, and the hermetic path bypasses the launcher.
        "cli_command": _HERMETIC_CLI_COMMAND if hermetic(caller_repository, version) else "",
        "ts_mutation_adapter_args": (
            _HERMETIC_TS_ADAPTER_ARGS if hermetic(caller_repository, version) else ""
        ),
    }


def _output_delimiter(value: str) -> str:
    """A heredoc delimiter that appears on no line of `value`. Derived from the value's
    hash so it's deterministic, then bumped until it can't collide with the content."""
    base = "ghadelimiter_" + hashlib.sha256(value.encode()).hexdigest()[:32]
    delim = base
    body = value.split("\n")
    counter = 0
    while delim in body:
        counter += 1
        delim = f"{base}_{counter}"
    return delim


def render_github_output(outputs: dict[str, str]) -> str:
    """Render `outputs` as the text appended to `GITHUB_OUTPUT`. A single-line value is a
    `name=value` line; a value carrying a newline (a `build_command` declared as a TOML
    multi-line string) uses the runner's heredoc form — `name<<DELIM`, the value, then
    `DELIM` on its own line — with a collision-free delimiter. A raw `name=value` line for a
    multi-line value would corrupt the file: the embedded newline ends the file-command line
    early and the remaining lines parse as bogus outputs.
    """
    lines: list[str] = []
    for name, value in outputs.items():
        if "\n" in value:
            delim = _output_delimiter(value)
            lines.append(f"{name}<<{delim}")
            lines.append(value)
            lines.append(delim)
        else:
            lines.append(f"{name}={value}")
    return "".join(f"{line}\n" for line in lines)


def main() -> int:
    languages = os.environ.get("LANGUAGES", "")
    scan_path = os.environ.get("SCAN_PATH", ".")
    config_input = os.environ.get("CONFIG", _CONFIG_DEFAULT)
    caller_repository = os.environ.get("CALLER_REPOSITORY", "")
    version = os.environ.get("VERSION", "")
    outputs = compute_outputs(
        languages,
        scan_path,
        config_input=config_input,
        caller_repository=caller_repository,
        version=version,
    )

    github_output = os.environ.get("GITHUB_OUTPUT")
    if github_output:
        with open(github_output, "a", encoding="utf-8") as handle:
            handle.write(render_github_output(outputs))
    summary = ", ".join(f"{name} {value}" for name, value in outputs.items())
    print(f"languages='{languages}' under '{scan_path}' -> {summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
