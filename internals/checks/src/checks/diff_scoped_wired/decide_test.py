"""Colocated unit tests for the diff-scoped-wired decision (isolation — pure, no I/O).

A fixture missing each `--base` check in turn pins the `and` joining them, and a fixture with
each check present but *without* `--base` pins the regexes (the `.*--base` tail, not the bare
command name).
"""
from checks.diff_scoped_wired.decide import wires_diff_scoped

BOTH = "run: unit colocated-test --base $BASE\nrun: unit coverage --base $BASE\n"
ONLY_CO_CHANGE = "run: unit colocated-test --base $BASE\nrun: unit coverage --language py\n"
ONLY_COVERAGE = "run: unit colocated-test --language py\nrun: unit coverage --base $BASE\n"
NEITHER_BASE = "run: unit colocated-test --language py\nrun: unit coverage --language py\n"


def test_true_when_both_base_checks_are_present():
    assert wires_diff_scoped(BOTH) is True


def test_false_when_only_the_co_change_check_is_present():
    assert wires_diff_scoped(ONLY_CO_CHANGE) is False


def test_false_when_only_the_changed_line_coverage_check_is_present():
    assert wires_diff_scoped(ONLY_COVERAGE) is False


def test_false_when_both_commands_run_without_the_base_flag():
    assert wires_diff_scoped(NEITHER_BASE) is False


def test_false_on_empty_text():
    assert wires_diff_scoped("") is False
