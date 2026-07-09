"""Integration tests for the ``testing_conventions`` pytest plugin.

Each test drives a real sub-pytest (``runpytest_subprocess``) over a tiny
project, so the installed ``pytest11`` entry point auto-loads the plugin exactly
as a consumer's ``pip install``ed run would. They pin the contract: our coverage
floor is applied by default, and any value the consumer sets themselves wins.
"""
import textwrap

WIDGET = """
def classify(n):
    if n > 0:
        return "pos"
    return "nonpos"
"""

WIDGET_TEST = """
from pkg.widget import classify


def test_pos():
    assert classify(5) == "pos"
"""


def _make_project(pytester, pyproject=None):
    pkg = pytester.path / "pkg"
    pkg.mkdir()
    (pkg / "__init__.py").write_text("")
    (pkg / "widget.py").write_text(textwrap.dedent(WIDGET))
    (pkg / "widget_test.py").write_text(textwrap.dedent(WIDGET_TEST))
    if pyproject is not None:
        (pytester.path / "pyproject.toml").write_text(textwrap.dedent(pyproject))


def test_applies_the_floor_by_default(pytester):
    # No coverage config: the plugin supplies branch on, fail_under=100, and omits
    # the test file. widget.py's `nonpos` branch is uncovered, so the floor fails.
    _make_project(pytester)
    result = pytester.runpytest_subprocess("--cov=pkg", "-q")
    out = "\n".join(result.stdout.lines)
    assert result.ret != 0, out
    assert "Branch" in out, out  # branch coverage is on
    assert "widget_test.py" not in out, out  # test files omitted from the report


def test_consumer_fail_under_wins(pytester):
    # The consumer lowers the floor; their value must be honored, so the run passes.
    _make_project(
        pytester,
        pyproject="""
        [tool.coverage.report]
        fail_under = 50
        """,
    )
    result = pytester.runpytest_subprocess("--cov=pkg")
    assert result.ret == 0, "\n".join(result.stdout.lines)


def test_consumer_branch_off_wins(pytester):
    # The consumer turns branch coverage off; the plugin must not force it on.
    _make_project(
        pytester,
        pyproject="""
        [tool.coverage.run]
        branch = false
        """,
    )
    result = pytester.runpytest_subprocess("--cov=pkg")
    out = "\n".join(result.stdout.lines)
    assert "Branch" not in out, out


def test_consumer_omit_wins(pytester):
    # The consumer sets their own omit; the plugin must not add ours on top, so the
    # test file is measured (and reported) again.
    _make_project(
        pytester,
        pyproject="""
        [tool.coverage.run]
        omit = ["nothing.py"]
        """,
    )
    result = pytester.runpytest_subprocess("--cov=pkg", "-q")
    out = "\n".join(result.stdout.lines)
    assert "widget_test.py" in out, out


def test_noop_without_coverage(pytester):
    # No --cov: a plain pytest run is untouched and passes.
    _make_project(pytester)
    result = pytester.runpytest_subprocess()
    assert result.ret == 0, "\n".join(result.stdout.lines)
    result.assert_outcomes(passed=1)
