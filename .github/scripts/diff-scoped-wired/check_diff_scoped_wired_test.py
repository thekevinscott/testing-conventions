"""Colocated unit tests for check_diff_scoped_wired.

Unit-level: the pure text inspection over crafted constants (no I/O). Both diff-scoped checks are
required, so each is dropped independently to prove the AND logic. `main` and the `__main__`
guard are covered by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_diff_scoped_wired as m  # noqa: E402

CO_CHANGE = 'run: unit colocated-test --language python --base "$BASE" "$PATH"\n'
CHANGED_LINE = 'run: unit coverage --language python --base "$BASE" "$PATH"\n'


def test_both_checks_present_reports_no_missing_wiring():
    assert m.find_missing_wiring(CO_CHANGE + CHANGED_LINE) is None


def test_missing_co_change_check_is_reported():
    msg = m.find_missing_wiring(CHANGED_LINE)
    assert msg is not None
    assert "#172" in msg


def test_missing_changed_line_coverage_check_is_reported():
    assert m.find_missing_wiring(CO_CHANGE) is not None


def test_neither_check_present_is_reported():
    assert m.find_missing_wiring("run: unit coverage --language python\n") is not None


def test_coverage_without_base_flag_does_not_satisfy_the_check():
    # `unit coverage` alone (whole-tree) is not the changed-line variant.
    assert m.find_missing_wiring(CO_CHANGE + "run: unit coverage --language python\n") is not None


def test_path_from_argv_uses_default_when_no_argument():
    assert m.path_from_argv(["prog"], "the-default") == "the-default"


def test_path_from_argv_prefers_the_explicit_argument():
    assert m.path_from_argv(["prog", "other.yml"], "the-default") == "other.yml"
