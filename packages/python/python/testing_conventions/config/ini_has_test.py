"""Unit tests for the ini-source read: presence, absence, and a malformed file."""
from testing_conventions.config.ini_has import ini_has


def test_ini_has_reports_a_key_the_section_sets(tmp_path):
    path = tmp_path / ".coveragerc"
    path.write_text("[run]\nbranch = true\n")
    assert ini_has(str(path), ["run"], "branch") is True


def test_ini_has_reports_a_key_the_section_omits(tmp_path):
    path = tmp_path / ".coveragerc"
    path.write_text("[run]\nbranch = true\n")
    assert ini_has(str(path), ["run"], "omit") is False


def test_ini_has_is_false_for_a_section_the_file_omits(tmp_path):
    path = tmp_path / ".coveragerc"
    path.write_text("[run]\nbranch = true\n")
    assert ini_has(str(path), ["report"], "branch") is False


def test_ini_has_on_a_malformed_file_is_false(tmp_path):
    path = tmp_path / "bad.cfg"
    path.write_text("no section header here\n")
    assert ini_has(str(path), ["run"], "branch") is False
