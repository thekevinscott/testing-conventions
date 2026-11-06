"""Unit tests for finding a mutated module's colocated test."""
from testing_conventions.mutation.colocated_test_path import colocated_test_path


def test_the_sibling_named_after_the_module_is_its_colocated_test(tmp_path):
    (tmp_path / "calc.py").write_text("")
    (tmp_path / "calc_test.py").write_text("")
    assert colocated_test_path(str(tmp_path / "calc.py")) == tmp_path / "calc_test.py"


def test_a_module_with_no_sibling_test_has_none(tmp_path):
    (tmp_path / "calc.py").write_text("")
    (tmp_path / "test_calc.py").write_text("")
    assert colocated_test_path(tmp_path / "calc.py") is None


def test_a_directory_named_like_the_test_is_not_a_test(tmp_path):
    (tmp_path / "calc_test.py").mkdir()
    assert colocated_test_path(tmp_path / "calc.py") is None
