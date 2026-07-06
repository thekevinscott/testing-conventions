#!/usr/bin/env python3
"""Assert the `detect` action's outputs for the monorepo TS fixture (#277, epic #302).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `detect-package-root-ts` job of the self-test
(.github/workflows/testing-conventions-selftest.yml). That job runs `./.github/actions/detect`
against `.github/selftest/monorepo/packages/ts/src` with only `languages`/`path`, then checks
that package-root discovery (#277) resolves the fixture's own directory, npm as the TS package
manager, no Rust provisioning, and the fixture's own `testing-conventions.toml` as config.

The assertion lives here — rather than inline in the workflow YAML — so it carries real
colocated and e2e tests. Inline `run:` bash is untested prose, and the workflow's `${{ }}`
templating is applied to the `run:` text before the shell sees it, which can silently rewrite a
literal expression. The detect outputs arrive as CLI arguments (never an env side-channel).
"""
import sys
from typing import Optional

# The outputs `./.github/actions/detect` must produce for the monorepo TS fixture (#277).
EXPECTED_PACKAGE_ROOT = ".github/selftest/monorepo/packages/ts"
EXPECTED_TS_PACKAGE_MANAGER = "npm"
EXPECTED_PROVISION_RUST = "false"
EXPECTED_CONFIG = ".github/selftest/monorepo/packages/ts/testing-conventions.toml"


def evaluate(
    package_root: str, ts_package_manager: str, provision_rust: str, config: str
) -> Optional[str]:
    """Return an error message for the first wrong detect output, else None.

    Pure: pairs each of the four detect outputs with the value package-root discovery must
    produce for the TS fixture, and returns that pair's message on the first mismatch.
    """
    checks = (
        (
            package_root,
            EXPECTED_PACKAGE_ROOT,
            "expected the ts fixture's own directory as package_root, "
            f"got {package_root}",
        ),
        (
            ts_package_manager,
            EXPECTED_TS_PACKAGE_MANAGER,
            "expected ts_package_manager=npm (package-lock.json, no packageManager field), "
            f"got {ts_package_manager}",
        ),
        (
            provision_rust,
            EXPECTED_PROVISION_RUST,
            "expected provision_rust=false (no Cargo.toml/maturin/napi), "
            f"got {provision_rust}",
        ),
        (
            config,
            EXPECTED_CONFIG,
            "expected the ts fixture's own testing-conventions.toml as config, "
            f"got {config}",
        ),
    )
    for actual, expected, message in checks:
        if actual != expected:
            return message
    return None


def main(argv: list) -> int:
    try:
        package_root, ts_package_manager, provision_rust, config = argv[1:]
    except ValueError:
        print(
            "::error::usage: check_detect_package_root_ts.py "
            "<package_root> <ts_package_manager> <provision_rust> <config>"
        )
        return 1
    error = evaluate(package_root, ts_package_manager, provision_rust, config)
    if error:
        print(f"::error::{error}")
        return 1
    print(
        f"package_root={package_root} ts_package_manager={ts_package_manager} "
        f"provision_rust={provision_rust} config={config}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
