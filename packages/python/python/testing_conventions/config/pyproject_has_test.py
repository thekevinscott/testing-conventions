"""Unit tests for the TOML-source read: presence, absence, and malformed TOML."""
from testing_conventions.config.pyproject_has import pyproject_has


def test_pyproject_has_reports_a_key_the_table_sets(tmp_path):
    path = tmp_path / "pyproject.toml"
    path.write_text("[tool.coverage.report]\nfail_under = 90\n")
    assert pyproject_has(str(path), "report", "fail_under") is True


def test_pyproject_has_reports_a_key_the_table_omits(tmp_path):
    path = tmp_path / "pyproject.toml"
    path.write_text("[tool.coverage.report]\nfail_under = 90\n")
    assert pyproject_has(str(path), "run", "branch") is False


def test_pyproject_has_on_malformed_toml_is_false(tmp_path):
    path = tmp_path / "pyproject.toml"
    path.write_text("= broken =\n")
    assert pyproject_has(str(path), "report", "fail_under") is False
