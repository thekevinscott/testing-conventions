"""Unit tests for the cosmic-ray → normalized-schema mapping (#248)."""
from types import SimpleNamespace

from testing_conventions.mutation.normalize import normalize


def _mutation(module_path="calc.py", start_pos=(6, 4), operator_name="core/Op"):
    return SimpleNamespace(
        module_path=module_path, start_pos=start_pos, operator_name=operator_name
    )


def _result(test_outcome):
    return SimpleNamespace(test_outcome=test_outcome)


def test_survived_maps_across_with_location_and_operator():
    mutant = normalize(_mutation(), _result("survived"))
    assert mutant == {"file": "calc.py", "line": 6, "status": "survived", "mutator": "core/Op"}


def test_killed_maps_to_killed():
    assert normalize(_mutation(), _result("killed"))["status"] == "killed"


def test_incompetent_maps_to_compile_error():
    assert normalize(_mutation(), _result("incompetent"))["status"] == "compile_error"


def test_enum_like_outcome_reads_its_value():
    assert normalize(_mutation(), _result(SimpleNamespace(value="survived")))["status"] == "survived"


def test_missing_outcome_is_skipped():
    assert normalize(_mutation(), _result(None)) is None


def test_unrecognized_outcome_is_skipped():
    assert normalize(_mutation(), _result("bogus")) is None


def test_backslash_separators_are_normalized():
    mutant = normalize(_mutation(module_path="pkg\\calc.py"), _result("survived"))
    assert mutant["file"] == "pkg/calc.py"
