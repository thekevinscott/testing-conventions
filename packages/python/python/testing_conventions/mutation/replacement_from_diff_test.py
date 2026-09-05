"""Unit tests for reading the mutated source out of a work result's diff."""
from testing_conventions.mutation.replacement_from_diff import replacement_from_diff

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


def test_the_added_line_is_the_replacement():
    assert replacement_from_diff(DIFF) == "return a >> b"


def test_no_diff_yields_none():
    assert replacement_from_diff(None) is None


def test_the_diff_header_is_never_read_as_a_replacement():
    header_only = "--- mutation diff ---\n--- a/calc.py\n+++ b/calc.py\n"
    assert replacement_from_diff(header_only) is None


def test_a_removal_only_diff_yields_none():
    removal = "@@ -1,3 +1,2 @@\n @decorate\n-    return a + b\n"
    assert replacement_from_diff(removal) is None


def test_an_unindented_added_line_keeps_its_first_character():
    assert replacement_from_diff("@@ -1,2 +1,2 @@\n-x = 1\n+y = 2\n") == "y = 2"


def test_a_multi_line_diff_names_every_added_line():
    multi = "@@ -1,3 +1,4 @@\n-    return a + b\n+    if a:\n+        return b\n"
    assert replacement_from_diff(multi) == "if a:\nreturn b"
