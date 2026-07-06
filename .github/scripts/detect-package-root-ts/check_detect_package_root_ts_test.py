"""Colocated unit tests for check_detect_package_root_ts.

Unit-level: the pure `evaluate` decision, exercised in isolation (no file reads, no subprocess,
no network). The single `actual != expected` comparison is pinned from every side — an equal
input, a lexically greater and a lexically lesser wrong input, and a value-equal-but-distinct
input — so a mutation to any other comparison operator (`<`, `>`, `<=`, `>=`, `==`, `is`,
`is not`) changes an asserted outcome. `main` and the `__main__` guard are covered by the e2e
suite under `tests/`.
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


def _package_root_message(value):
    return (
        "expected the ts fixture's own directory as package_root, "
        f"got {value}"
    )


def test_evaluate_accepts_the_expected_outputs():
    assert m.evaluate(*GOOD) is None


def test_evaluate_flags_package_root_greater_than_expected():
    greater = m.EXPECTED_PACKAGE_ROOT + "/src"
    assert greater > m.EXPECTED_PACKAGE_ROOT
    assert m.evaluate(greater, GOOD[1], GOOD[2], GOOD[3]) == _package_root_message(greater)


def test_evaluate_flags_package_root_less_than_expected():
    lesser = ".github/selftest/monorepo/packages"
    assert lesser < m.EXPECTED_PACKAGE_ROOT
    assert m.evaluate(lesser, GOOD[1], GOOD[2], GOOD[3]) == _package_root_message(lesser)


def test_evaluate_accepts_a_value_equal_but_distinct_package_root():
    distinct = "".join(list(m.EXPECTED_PACKAGE_ROOT))
    assert distinct == m.EXPECTED_PACKAGE_ROOT
    assert m.evaluate(distinct, GOOD[1], GOOD[2], GOOD[3]) is None


def test_evaluate_flags_wrong_package_manager():
    assert m.evaluate(GOOD[0], "pnpm", GOOD[2], GOOD[3]) == (
        "expected ts_package_manager=npm (package-lock.json, no packageManager field), "
        "got pnpm"
    )


def test_evaluate_flags_wrong_provision_rust():
    assert m.evaluate(GOOD[0], GOOD[1], "true", GOOD[3]) == (
        "expected provision_rust=false (no Cargo.toml/maturin/napi), got true"
    )


def test_evaluate_flags_wrong_config():
    assert m.evaluate(GOOD[0], GOOD[1], GOOD[2], "testing-conventions.toml") == (
        "expected the ts fixture's own testing-conventions.toml as config, "
        "got testing-conventions.toml"
    )
