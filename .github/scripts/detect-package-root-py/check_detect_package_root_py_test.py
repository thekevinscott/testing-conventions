"""Colocated unit tests for check_detect_package_root_py.

Unit-level: the pure `evaluate` decision, exercised in isolation (no file reads, no subprocess,
no network). `main` and the `__main__` guard are covered by the e2e suite under `tests/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_detect_package_root_py as m  # noqa: E402

GOOD = (m.EXPECTED_PACKAGE_ROOT, m.EXPECTED_PYTHON_ENV, m.EXPECTED_CONFIG)


def test_evaluate_accepts_the_expected_outputs():
    assert m.evaluate(*GOOD) is None


def test_evaluate_rejects_wrong_package_root():
    error = m.evaluate(".github/selftest/monorepo/packages/py/src", GOOD[1], GOOD[2])
    assert error == (
        "expected the py fixture's own directory as package_root, "
        "got .github/selftest/monorepo/packages/py/src"
    )


def test_evaluate_rejects_wrong_python_env():
    error = m.evaluate(GOOD[0], "poetry", GOOD[2])
    assert error == "expected python_env=uv ([project] table present), got poetry"


def test_evaluate_rejects_wrong_config():
    error = m.evaluate(GOOD[0], GOOD[1], "testing-conventions.toml")
    assert error == (
        "expected the py fixture's own testing-conventions.toml as config, "
        "got testing-conventions.toml"
    )
