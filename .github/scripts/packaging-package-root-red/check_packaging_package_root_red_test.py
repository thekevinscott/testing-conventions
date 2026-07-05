"""Colocated unit tests for check_packaging_package_root_red.

Unit-level: the pure decision functions, exercised in isolation (no subprocess, no mocks). The
`main` orchestration is covered by the integration suite with the boundary mocked, and the whole
entry point by the e2e suite against a benign command — both under `tests/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_packaging_package_root_red as m  # noqa: E402


def test_expect_failure_is_none_when_the_command_failed():
    # Any non-zero exit is a pass for a red-path check — including a signal-kill (negative code).
    assert m.expect_failure(1) is None
    assert m.expect_failure(2) is None
    assert m.expect_failure(-9) is None


def test_expect_failure_reports_when_the_command_passed():
    assert m.expect_failure(0) == "the command exited 0, but a non-zero (failing) exit was required"


def test_expect_success_is_none_when_the_command_passed():
    assert m.expect_success(0) is None


def test_expect_success_reports_when_the_command_failed():
    assert m.expect_success(3) == "the command exited 3, but a zero (passing) exit was required"
    assert m.expect_success(-9) == "the command exited -9, but a zero (passing) exit was required"


def test_evaluate_routes_to_expect_failure_for_a_red_check():
    # expect_fail=True: a zero exit is the violation, a non-zero exit is clean.
    assert m.evaluate(True, 0) == m.expect_failure(0)
    assert m.evaluate(True, 1) is None


def test_evaluate_routes_to_expect_success_for_a_clean_check():
    assert m.evaluate(False, 1) == m.expect_success(1)
    assert m.evaluate(False, 0) is None


def test_parse_returns_the_hardcoded_checks_with_no_arguments():
    assert m.parse(["check_packaging_package_root_red.py"]) is m.CHECKS


def test_parse_builds_a_single_cli_check_from_arguments():
    # The length boundary (one arg -> CHECKS; two or more -> a cli check) and the mode routing.
    assert m.parse(["prog", "fail"]) == [([], True, "cli")]
    assert m.parse(["prog", "fail", "npx", "x"]) == [(["npx", "x"], True, "cli")]
    assert m.parse(["prog", "pass", "true"]) == [(["true"], False, "cli")]


def test_checks_are_the_expected_invocations():
    assert m.CHECKS == [
    (
        ["npx", "-y", "testing-conventions", "packaging", ".github/selftest/packaging-package-root/red/dist/packaging-package-root-fixture-0.0.0.tgz", "--language", "typescript"],
        True,
        "red per-package dist tarball trips the packaging check",
    ),
]
