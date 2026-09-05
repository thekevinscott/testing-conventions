import json
from pathlib import Path

from build_command_language import build_command_language
from cargo_workspace import cargo_workspace_root
from derive_build_command import derive_build_command
from derive_cargo_target_dir import derive_cargo_target_dir
from derive_config import CONFIG_DEFAULT, derive_config
from derive_package_root import derive_package_root
from derive_packaging import derive_packaging
from e2e_scope_flags import derive_e2e_exclude, derive_e2e_extra_scope
from eligible import eligible
from file_presence import has_rust_crate, has_source
from has_attestation import has_attestation
from has_dist import has_dist
from hermetic import HERMETIC_CLI_COMMAND, HERMETIC_TS_ADAPTER_ARGS, hermetic
from primary_language import primary_language
from provision_rust import provision_rust
from python_env import python_env
from ts_package_manager import ts_package_manager
from ts_pnpm_version import ts_pnpm_version


def _to_json(languages: list[str]) -> str:
    """Compact JSON array, matching what the matrix `fromJSON(...)` consumes (`[]` when empty)."""
    return json.dumps(languages, separators=(",", ":"))


def compute_outputs(
    languages_input: str,
    scan_root: str,
    repo_root: str = ".",
    config_input: str = CONFIG_DEFAULT,
    caller_repository: str = "",
    version: str = "",
) -> dict[str, str]:
    """The detected sets, as `name -> value` strings for GITHUB_OUTPUT.

    See `docs/internals/repo.md` for what each matrix and package-root output means.
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
    workspace_root = cargo_workspace_root(package_root, repo)
    if workspace_root is not None:
        try:
            workspace_root_rel = workspace_root.relative_to(repo.resolve())
        except ValueError:
            workspace_root_rel = Path(".")
    else:
        workspace_root_rel = None
    cargo_target_dir = derive_cargo_target_dir(package_root_rel, workspace_root_rel)
    return {
        "languages": _to_json(present),
        "colocated_test_languages": _to_json(with_rust),
        "integration_lint_languages": _to_json(with_rust),
        "isolation_languages": _to_json(with_rust),
        "static_languages": _to_json(with_rust),
        "one_function_languages": _to_json(with_rust),
        "coverage_languages": _to_json(with_rust),
        "mutation_languages": _to_json(with_rust),
        "packaging_dist": "true" if has_dist(package_root) else "false",
        "e2e_attestation": "true" if has_attestation(package_root) else "false",
        "package_root": str(package_root_rel),
        "ts_package_manager": ts_package_manager(package_root),
        "ts_pnpm_version": ts_pnpm_version(package_root),
        "python_env": python_env(package_root),
        "provision_rust": provision_rust(package_root),
        "cargo_target_dir": cargo_target_dir,
        "config": config,
        "build_command": derive_build_command(config, bc_language),
        "packaging_build": packaging_build,
        "packaging_language": primary if packaging_build else "",
        "e2e_extra_scope": derive_e2e_extra_scope(config),
        "e2e_exclude": derive_e2e_exclude(config),
        "cli_command": HERMETIC_CLI_COMMAND if hermetic(caller_repository, version) else "",
        "ts_mutation_adapter_args": (
            HERMETIC_TS_ADAPTER_ARGS if hermetic(caller_repository, version) else ""
        ),
    }
