"""The detect-package-root-py decision — repo-only (#277 #327)."""
from __future__ import annotations

from typing import Optional

from checks.config import (
    PY_FIXTURE_CONFIG,
    PY_FIXTURE_PACKAGE_ROOT,
    PY_FIXTURE_PYTHON_ENV,
)


def evaluate(package_root: str, python_env: str, config: str) -> Optional[str]:
    """Return an error message for the first wrong detect output, else None.

    Pure: pairs each of the three detect outputs with the value package-root discovery must
    produce for the Python fixture, and returns that pair's message on the first mismatch.
    """
    checks = (
        (
            package_root,
            PY_FIXTURE_PACKAGE_ROOT,
            f"expected the py fixture's own directory as package_root, got {package_root}",
        ),
        (
            python_env,
            PY_FIXTURE_PYTHON_ENV,
            f"expected python_env=uv ([project] table present), got {python_env}",
        ),
        (
            config,
            PY_FIXTURE_CONFIG,
            f"expected the py fixture's own testing-conventions.toml as config, got {config}",
        ),
    )
    for actual, expected, message in checks:
        if actual != expected:
            return message
    return None
