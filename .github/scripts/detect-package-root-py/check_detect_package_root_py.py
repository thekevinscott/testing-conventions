#!/usr/bin/env python3
"""Assert the `detect` action's outputs for the monorepo uv Python fixture (#277, epic #302).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `detect-package-root-py` job of the self-test
(.github/workflows/testing-conventions-selftest.yml). That job runs `./.github/actions/detect`
against `.github/selftest/monorepo/packages/py/src` with only `languages`/`path`, then checks
that package-root discovery (#277) resolves the fixture's own directory, uv as the Python
environment, and the fixture's own `testing-conventions.toml` as config.

The assertion lives here — rather than inline in the workflow YAML — so it carries real
colocated and e2e tests. Inline `run:` bash is untested prose, and the workflow's `${{ }}`
templating is applied to the `run:` text before the shell sees it, which can silently rewrite a
literal expression. The detect outputs arrive as CLI arguments (never an env side-channel).
"""
import sys
from typing import Optional

# The outputs `./.github/actions/detect` must produce for the monorepo Python fixture (#277).
EXPECTED_PACKAGE_ROOT = ".github/selftest/monorepo/packages/py"
EXPECTED_PYTHON_ENV = "uv"
EXPECTED_CONFIG = ".github/selftest/monorepo/packages/py/testing-conventions.toml"


def evaluate(package_root: str, python_env: str, config: str) -> Optional[str]:
    """Return an error message for the first wrong detect output, else None.

    Pure: pairs each of the three detect outputs with the value package-root discovery must
    produce for the Python fixture, and returns that pair's message on the first mismatch.
    """
    checks = (
        (
            package_root,
            EXPECTED_PACKAGE_ROOT,
            "expected the py fixture's own directory as package_root, "
            f"got {package_root}",
        ),
        (
            python_env,
            EXPECTED_PYTHON_ENV,
            f"expected python_env=uv ([project] table present), got {python_env}",
        ),
        (
            config,
            EXPECTED_CONFIG,
            "expected the py fixture's own testing-conventions.toml as config, "
            f"got {config}",
        ),
    )
    for actual, expected, message in checks:
        if actual != expected:
            return message
    return None


def main(argv: list) -> int:
    try:
        package_root, python_env, config = argv[1:]
    except ValueError:
        print(
            "::error::usage: check_detect_package_root_py.py "
            "<package_root> <python_env> <config>"
        )
        return 1
    error = evaluate(package_root, python_env, config)
    if error:
        print(f"::error::{error}")
        return 1
    print(f"package_root={package_root} python_env={python_env} config={config}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
