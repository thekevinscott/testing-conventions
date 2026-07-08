"""Unit tests for the coverage-default decision."""
from testing_conventions.pytest_plugin.plan import FAIL_UNDER, OMIT, decide


def _user_set(*pairs):
    """A fake ``user_set`` that reports True only for the given (key, table) pairs."""
    wanted = set(pairs)
    return lambda start, cov_config, key, table: (key, table) in wanted


def test_all_defaults_apply_when_unconfigured():
    d = decide([], "/x", None, _user_set())
    assert d.branch is True
    assert d.fail_under is True
    assert d.omit is True


def test_cli_cov_branch_suppresses_only_branch():
    d = decide(["--cov-branch"], "/x", None, _user_set())
    assert d.branch is False
    assert d.fail_under is True


def test_cli_cov_fail_under_suppresses_only_fail_under():
    d = decide(["--cov-fail-under=80"], "/x", None, _user_set())
    assert d.fail_under is False
    assert d.branch is True


def test_config_branch_suppresses_only_branch():
    d = decide([], "/x", None, _user_set(("branch", "run")))
    assert d.branch is False
    assert d.fail_under is True


def test_config_fail_under_suppresses_only_fail_under():
    d = decide([], "/x", None, _user_set(("fail_under", "report")))
    assert d.fail_under is False
    assert d.branch is True


def test_omit_suppressed_by_run_omit():
    assert decide([], "/x", None, _user_set(("omit", "run"))).omit is False


def test_omit_suppressed_by_report_omit():
    assert decide([], "/x", None, _user_set(("omit", "report"))).omit is False


def test_recommended_values():
    assert FAIL_UNDER == 100.0
    assert OMIT == ["*_test.py", "*/conftest.py", "conftest.py"]
