"""The detect-package-root-ts decision — repo-only (#277 #327)."""
from __future__ import annotations

from typing import Optional

from checks.config import (
    TS_FIXTURE_CONFIG,
    TS_FIXTURE_PACKAGE_MANAGER,
    TS_FIXTURE_PACKAGE_ROOT,
    TS_FIXTURE_PROVISION_RUST,
)


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
            TS_FIXTURE_PACKAGE_ROOT,
            f"expected the ts fixture's own directory as package_root, got {package_root}",
        ),
        (
            ts_package_manager,
            TS_FIXTURE_PACKAGE_MANAGER,
            "expected ts_package_manager=npm (package-lock.json, no packageManager field), "
            f"got {ts_package_manager}",
        ),
        (
            provision_rust,
            TS_FIXTURE_PROVISION_RUST,
            f"expected provision_rust=false (no Cargo.toml/maturin/napi), got {provision_rust}",
        ),
        (
            config,
            TS_FIXTURE_CONFIG,
            f"expected the ts fixture's own testing-conventions.toml as config, got {config}",
        ),
    )
    for actual, expected, message in checks:
        if actual != expected:
            return message
    return None
