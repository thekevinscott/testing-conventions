"""Unit tests for the coverage-config detection (#218).

These pin the precedence contract — the plugin applies a default only where the
consumer set nothing — across every config source coverage.py reads.
"""
from testing_conventions._config import _ini_has, _pyproject_has, user_set


def test_pyproject_table_is_detected(tmp_path):
    (tmp_path / "pyproject.toml").write_text("[tool.coverage.report]\nfail_under = 50\n")
    assert user_set(tmp_path, None, "fail_under", "report") is True


def test_pyproject_without_the_key_is_not_detected(tmp_path):
    (tmp_path / "pyproject.toml").write_text("[tool.coverage.run]\nbranch = true\n")
    assert user_set(tmp_path, None, "fail_under", "report") is False


def test_coveragerc_bare_section_is_detected(tmp_path):
    (tmp_path / ".coveragerc").write_text("[run]\nbranch = true\n")
    assert user_set(tmp_path, None, "branch", "run") is True


def test_setup_cfg_prefixed_section_is_detected(tmp_path):
    (tmp_path / "setup.cfg").write_text("[coverage:run]\nomit =\n    a.py\n")
    assert user_set(tmp_path, None, "omit", "run") is True


def test_tox_ini_prefixed_section_is_detected(tmp_path):
    (tmp_path / "tox.ini").write_text("[coverage:report]\nfail_under = 80\n")
    assert user_set(tmp_path, None, "fail_under", "report") is True


def test_explicit_relative_cov_config_is_read_as_bare_ini(tmp_path):
    (tmp_path / "cov.ini").write_text("[report]\nfail_under = 70\n")
    assert user_set(tmp_path, "cov.ini", "fail_under", "report") is True


def test_explicit_absolute_cov_config_is_read(tmp_path):
    cfg = tmp_path / "abs.ini"
    cfg.write_text("[run]\nbranch = true\n")
    assert user_set(tmp_path, str(cfg), "branch", "run") is True


def test_explicit_toml_cov_config_is_read_as_toml(tmp_path):
    (tmp_path / "cov.toml").write_text("[tool.coverage.run]\nbranch = true\n")
    assert user_set(tmp_path, "cov.toml", "branch", "run") is True


def test_setup_cfg_section_without_the_key_is_not_detected(tmp_path):
    (tmp_path / "setup.cfg").write_text("[coverage:run]\nbranch = true\n")
    assert user_set(tmp_path, None, "omit", "run") is False


def test_coveragerc_section_without_the_key_is_not_detected(tmp_path):
    (tmp_path / ".coveragerc").write_text("[run]\nbranch = true\n")
    assert user_set(tmp_path, None, "omit", "run") is False


def test_setting_in_a_parent_directory_is_detected(tmp_path):
    (tmp_path / ".coveragerc").write_text("[run]\nbranch = true\n")
    child = tmp_path / "pkg" / "sub"
    child.mkdir(parents=True)
    assert user_set(child, None, "branch", "run") is True


def test_nothing_set_anywhere_returns_false(tmp_path):
    child = tmp_path / "pkg"
    child.mkdir()
    assert user_set(child, None, "branch", "run") is False


def test_ini_has_reports_presence(tmp_path):
    path = tmp_path / ".coveragerc"
    path.write_text("[run]\nbranch = true\n")
    assert _ini_has(str(path), ["run"], "branch") is True
    assert _ini_has(str(path), ["run"], "omit") is False


def test_pyproject_has_reports_presence(tmp_path):
    path = tmp_path / "pyproject.toml"
    path.write_text("[tool.coverage.report]\nfail_under = 90\n")
    assert _pyproject_has(str(path), "report", "fail_under") is True
    assert _pyproject_has(str(path), "run", "branch") is False
