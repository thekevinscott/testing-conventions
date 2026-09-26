"""Colocated unit tests for the agents-md-size orchestration (isolation — injected git read and
size predicate).

Both collaborators are hand-rolled fakes, so the orchestration is exercised without a repo, a
subprocess, or a file. The fakes record what they saw, which pins that `run` threads its own
arguments through rather than reading some other root or some other budget.
"""
import inspect

from checks.agents_md_size.gate import BUDGET, run


def test_the_default_read_is_the_real_git_read():
    # Pinned by module and name rather than identity, so no collaborator is imported.
    defaults = {
        name: (parameter.default.__module__, parameter.default.__name__)
        for name, parameter in inspect.signature(run).parameters.items()
        if parameter.kind is inspect.Parameter.KEYWORD_ONLY and callable(parameter.default)
    }
    assert defaults == {
        "tracked_paths": ("checks.utils.tracked_paths", "tracked_paths"),
        "fits": ("checks.agents_md_size.agents_md_size", "agents_md_size"),
    }


def test_the_default_budget_is_the_documented_cap():
    assert inspect.signature(run).parameters["budget"].default == BUDGET
    assert BUDGET == 16384


def _run(root="root", paths=(), over=()):
    """Drive the gate over `paths`; every path in `over` is treated as exceeding the budget."""
    seen_roots, seen_sizes = [], []

    def tracked_paths(r):
        seen_roots.append(r)
        return list(paths)

    def fits(path, budget):
        seen_sizes.append((str(path), budget))
        return not any(str(path).endswith(name) for name in over)

    return run(root, tracked_paths=tracked_paths, fits=fits), seen_roots, seen_sizes


def test_a_tree_of_files_within_budget_is_a_pass(capsys):
    code, _, _ = _run(paths=["AGENTS.md"])
    assert code == 0
    assert "every instructions file fits the budget" in capsys.readouterr().out


def test_an_over_budget_file_fails_naming_the_budget_and_the_remedy(capsys):
    code, _, _ = _run(paths=["AGENTS.md"], over=["AGENTS.md"])
    out = capsys.readouterr().out
    assert code == 1
    assert "::error file=AGENTS.md::" in out
    assert "16384" in out
    assert "reference files or skills" in out


def test_the_failure_says_the_count_happens_after_imports_resolve(capsys):
    # A short index importing five files costs the same context as one long file.
    _run(paths=["AGENTS.md"], over=["AGENTS.md"])
    assert "after `@path` imports resolve" in capsys.readouterr().out


def test_multiple_over_budget_files_each_get_an_annotation(capsys):
    code, _, _ = _run(paths=["AGENTS.md", "pkg/CLAUDE.md"], over=["AGENTS.md", "CLAUDE.md"])
    out = capsys.readouterr().out
    assert code == 1
    assert "::error file=AGENTS.md::" in out
    assert "::error file=pkg/CLAUDE.md::" in out


def test_a_passing_run_reports_no_violations(capsys):
    code, _, _ = _run(paths=["AGENTS.md"])
    assert code == 0
    assert "::error" not in capsys.readouterr().out


def test_a_tree_with_no_instructions_file_is_a_pass(capsys):
    code, _, sizes = _run(paths=["README.md", "src/main.py"])
    assert code == 0
    assert sizes == [], "a file that is not instructions is never measured"


def test_the_read_is_asked_for_the_root_under_test():
    _, roots, _ = _run(root="pkg")
    assert roots == ["pkg"]


def test_each_file_is_measured_under_the_root():
    # The git read is relative to the root, so the size read has to be rejoined to it.
    _, _, sizes = _run(root="pkg", paths=["a/AGENTS.md"])
    assert [path for path, _ in sizes] == ["pkg/a/AGENTS.md"]


def test_the_budget_reaches_the_size_predicate():
    seen = []

    def fits(path, budget):
        seen.append(budget)
        return True

    run("root", budget=99, tracked_paths=lambda _: ["AGENTS.md"], fits=fits)
    assert seen == [99]


def test_the_budget_reaches_the_failure_message(capsys):
    run("root", budget=99, tracked_paths=lambda _: ["AGENTS.md"], fits=lambda *_: False)
    assert "99-character" in capsys.readouterr().out
