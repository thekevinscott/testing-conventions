"""Colocated unit tests for the fail-closed conclusion reading (isolation — pure dict in/out)."""
from checks.utils.verify_release.failed_suites import failed_suites


def test_failed_suites_labels_non_success_conclusions():
    assert failed_suites({"a.yml": "success", "b.yml": "failure"}) == ["b.yml (failure)"]


def test_failed_suites_names_a_missing_conclusion_rather_than_dropping_it():
    # `conclusion or 'no conclusion'`, not `and`: a None conclusion (cancelled/timed-out run) is a
    # failure and must be named, with a readable placeholder.
    assert failed_suites({"a.yml": None}) == ["a.yml (no conclusion)"]


def test_failed_suites_is_empty_when_every_suite_succeeded():
    # A freshly-built (non-interned) "success" string: `!= "success"`, not `is not "success"` — an
    # identity mutant would treat the equal-but-not-identical value as a failure.
    success = "".join(["succ", "ess"])
    assert failed_suites({"a.yml": success, "b.yml": success}) == []
