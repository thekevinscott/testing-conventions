"""Unit tests for the cosmic-ray → normalized-schema mapping."""
from types import SimpleNamespace

from testing_conventions.mutation.normalize import normalize


def _mutation(module_path="calc.py", start_pos=(6, 4), operator_name="core/Op"):
    return SimpleNamespace(
        module_path=module_path, start_pos=start_pos, operator_name=operator_name
    )


DIFF = (
    "--- mutation diff ---\n"
    "--- a/calc.py\n"
    "+++ b/calc.py\n"
    "@@ -1,5 +1,5 @@\n"
    " def add(a, b):\n"
    "-    return a + b\n"
    "+    return a >> b\n"
    " \n"
)


def _result(test_outcome, diff=None):
    return SimpleNamespace(test_outcome=test_outcome, diff=diff)


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


def test_the_diff_carries_the_mutated_source_as_the_replacement():
    mutant = normalize(_mutation(), _result("survived", DIFF))
    assert mutant["replacement"] == "return a >> b"


def test_a_result_with_no_diff_omits_the_replacement():
    assert "replacement" not in normalize(_mutation(), _result("survived"))


def test_the_diff_header_is_never_read_as_a_replacement():
    header_only = "--- mutation diff ---\n--- a/calc.py\n+++ b/calc.py\n"
    assert "replacement" not in normalize(_mutation(), _result("survived", header_only))


def test_a_removal_only_mutation_omits_the_replacement():
    removal = "@@ -1,3 +1,2 @@\n @decorate\n-    return a + b\n"
    assert "replacement" not in normalize(_mutation(), _result("survived", removal))


def test_an_unindented_added_line_keeps_its_first_character():
    diff = "@@ -1,2 +1,2 @@\n-x = 1\n+y = 2\n"
    assert normalize(_mutation(), _result("survived", diff))["replacement"] == "y = 2"


def test_a_multi_line_mutation_names_every_added_line():
    multi = "@@ -1,3 +1,4 @@\n-    return a + b\n+    if a:\n+        return b\n"
    mutant = normalize(_mutation(), _result("survived", multi))
    assert mutant["replacement"] == "if a:\nreturn b"
