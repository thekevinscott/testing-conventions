"""Colocated unit tests for the path-length-gate orchestration (isolation — injected git read).

The git read is injected as a hand-rolled fake, so the orchestration is exercised without a repo
or a subprocess. The fake records the root it saw, which pins that `run` threads its own argument
through rather than reading some other root.
"""
import inspect

from checks.path_length_gate.gate import run


def test_the_default_read_is_the_real_git_read():
    # Pinned by module and name rather than identity, so no git-read collaborator is imported.
    defaults = {
        name: (parameter.default.__module__, parameter.default.__name__)
        for name, parameter in inspect.signature(run).parameters.items()
        if parameter.kind is inspect.Parameter.KEYWORD_ONLY
    }
    assert defaults == {"tracked_paths": ("checks.path_length_gate.git_ops", "tracked_paths")}


def _run(root="root", paths=()):
    seen = []

    def tracked_paths(r):
        seen.append(r)
        return list(paths)

    return run(root, tracked_paths=tracked_paths), seen


def test_no_violations_is_a_pass(capsys):
    code, _ = _run(paths=["short.md"])
    assert code == 0
    assert "every tracked path fits the Windows checkout budget" in capsys.readouterr().out


def test_a_violation_fails_naming_the_path_length_and_budget(capsys):
    overlong = "a" * 205
    code, _ = _run(paths=[overlong])
    out = capsys.readouterr().out
    assert code == 1
    assert f"::error file={overlong}::" in out
    assert "205" in out
    assert "200" in out


def test_multiple_violations_each_get_an_annotation(capsys):
    first = "a" * 205
    second = "b" * 210
    code, _ = _run(paths=[first, second])
    out = capsys.readouterr().out
    assert code == 1
    assert f"::error file={first}::" in out
    assert f"::error file={second}::" in out


def test_a_passing_run_reports_no_violations(capsys):
    code, _ = _run(paths=["short.md"])
    assert code == 0
    assert "::error" not in capsys.readouterr().out


def test_the_read_is_asked_for_the_root_under_test():
    _, seen = _run(root="pkg")
    assert seen == ["pkg"]
