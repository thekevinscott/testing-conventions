"""Colocated unit test for CheckFailed — it prints a GitHub `::error::` annotation (isolation)."""
from checks.utils.check_failed import CheckFailed


def test_show_emits_a_github_error_annotation(capsys):
    CheckFailed("the wiring is missing").show()
    assert capsys.readouterr().out == "::error::the wiring is missing\n"
