"""Colocated unit tests for check_detect_package_root_ts.

Unit-level: the pure `evaluate` decision, exercised in isolation (no file reads, no subprocess,
no network). `main` and the `__main__` guard are covered by the e2e suite under `tests/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_detect_package_root_ts as m  # noqa: E402

GOOD = (
    m.EXPECTED_PACKAGE_ROOT,
    m.EXPECTED_TS_PACKAGE_MANAGER,
    m.EXPECTED_PROVISION_RUST,
    m.EXPECTED_CONFIG,
)


def test_evaluate_accepts_the_expected_outputs():
    assert m.evaluate(*GOOD) is None


def test_evaluate_rejects_wrong_package_root():
    error = m.evaluate(".github/selftest/monorepo/packages/ts/src", *GOOD[1:])
    assert error == (
        "expected the ts fixture's own directory as package_root, "
        "got .github/selftest/monorepo/packages/ts/src"
    )


def test_evaluate_rejects_wrong_package_manager():
    error = m.evaluate(GOOD[0], "pnpm", GOOD[2], GOOD[3])
    assert error == (
        "expected ts_package_manager=npm (package-lock.json, no packageManager field), "
        "got pnpm"
    )


def test_evaluate_rejects_wrong_provision_rust():
    error = m.evaluate(GOOD[0], GOOD[1], "true", GOOD[3])
    assert error == (
        "expected provision_rust=false (no Cargo.toml/maturin/napi), got true"
    )


def test_evaluate_rejects_wrong_config():
    error = m.evaluate(GOOD[0], GOOD[1], GOOD[2], "testing-conventions.toml")
    assert error == (
        "expected the ts fixture's own testing-conventions.toml as config, "
        "got testing-conventions.toml"
    )
