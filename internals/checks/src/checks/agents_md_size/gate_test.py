"""Colocated unit tests for the agents-md-size orchestration (isolation — injected reads).

Every read is injected as a hand-rolled fake, so the orchestration is exercised without a repo, a
subprocess, or a filesystem. The fakes record what they were asked for, which pins that `run`
threads its own arguments through rather than reading some other root or ref.
"""
import inspect
from types import SimpleNamespace

from checks.agents_md_size.gate import run

LIMITS = SimpleNamespace(warn_lines=10, warn_bytes=1000, max_lines=20, max_bytes=2000)


def _document(lines):
    return "".join("x\n" for _ in range(lines))


def _run(files, *, limits=LIMITS, base=None, changed=()):
    """Drive `run` over a dict of path -> text; return (exit code, what each fake was asked)."""
    asked = {"roots": [], "reads": [], "diffs": []}

    def tracked_paths(root):
        asked["roots"].append(root)
        return list(files)

    def read_text(root, path):
        asked["reads"].append((root, path))
        return files.get(path)

    def changed_paths(root, ref):
        asked["diffs"].append((root, ref))
        return list(changed)

    code = run(
        "repo",
        limits,
        base,
        tracked_paths=tracked_paths,
        read_text=read_text,
        changed_paths=changed_paths,
    )
    return code, asked


def test_the_default_reads_are_the_real_ones():
    # Pinned by module and name rather than identity, so no read collaborator is imported.
    defaults = {
        name: (parameter.default.__module__, parameter.default.__name__)
        for name, parameter in inspect.signature(run).parameters.items()
        if parameter.kind is inspect.Parameter.KEYWORD_ONLY
    }
    assert defaults == {
        "tracked_paths": ("checks.agents_md_size.git_ops", "tracked_paths"),
        "read_text": ("checks.agents_md_size.fs_ops", "read_text"),
        "changed_paths": ("checks.agents_md_size.diff_ops", "changed_paths"),
    }


def test_the_default_budgets_are_the_shipped_ones():
    # Pinned by module and name rather than identity, so no budget collaborator is imported.
    default = type(inspect.signature(run).parameters["limits"].default)
    assert (default.__module__, default.__name__) == ("checks.agents_md_size.limits", "Limits")


def test_a_tree_with_no_instructions_file_passes(capsys):
    code, _ = _run({"README.md": _document(500)})
    assert code == 0
    assert "every instructions file fits its budget" in capsys.readouterr().out


def test_a_small_instructions_file_passes(capsys):
    code, _ = _run({"AGENTS.md": _document(3)})
    assert code == 0
    assert "every instructions file fits its budget" in capsys.readouterr().out


def test_an_over_budget_file_fails_naming_its_size_and_budget(capsys):
    code, _ = _run({"AGENTS.md": _document(LIMITS.max_lines + 1)})
    out = capsys.readouterr().out
    assert code == 1
    assert "::error file=AGENTS.md::" in out
    assert f"{LIMITS.max_lines + 1} lines" in out
    assert f"{LIMITS.max_lines}-line / {LIMITS.max_bytes}-byte budget" in out
    assert "Move sections into reference files or skills" in out


def test_an_over_budget_file_names_every_member_of_its_closure(capsys):
    code, _ = _run(
        {"AGENTS.md": "@agents/style.md\n", "agents/style.md": _document(LIMITS.max_lines + 1)}
    )
    out = capsys.readouterr().out
    assert code == 1
    assert "Closure: AGENTS.md (1 lines), agents/style.md" in out
    assert f"({LIMITS.max_lines + 1} lines)" in out


def test_a_file_past_the_soft_budget_warns_without_failing(capsys):
    code, _ = _run({"AGENTS.md": _document(LIMITS.warn_lines + 1)})
    out = capsys.readouterr().out
    assert code == 0
    assert "::warning file=AGENTS.md::" in out
    assert f"{LIMITS.warn_lines}-line / {LIMITS.warn_bytes}-byte soft budget" in out


def test_a_warning_suppresses_the_everything_fits_message(capsys):
    _run({"AGENTS.md": _document(LIMITS.warn_lines + 1)})
    assert "every instructions file fits its budget" not in capsys.readouterr().out


def test_a_warning_beside_a_failure_still_fails(capsys):
    code, _ = _run(
        {
            "AGENTS.md": _document(LIMITS.warn_lines + 1),
            "pkg/AGENTS.md": _document(LIMITS.max_lines + 1),
        }
    )
    out = capsys.readouterr().out
    assert code == 1
    assert "::warning file=AGENTS.md::" in out
    assert "::error file=pkg/AGENTS.md::" in out


def test_a_passing_run_annotates_nothing(capsys):
    _run({"AGENTS.md": _document(3)})
    assert "::" not in capsys.readouterr().out


def test_the_tree_is_listed_for_the_root_under_test():
    _, asked = _run({"AGENTS.md": _document(3)})
    assert asked["roots"] == ["repo"]


def test_every_read_is_rooted_at_the_root_under_test():
    _, asked = _run({"AGENTS.md": "@a.md\n", "a.md": "aye\n"})
    assert asked["reads"] == [("repo", "AGENTS.md"), ("repo", "a.md")]


def test_a_file_that_is_not_an_instructions_file_is_never_read():
    _, asked = _run({"AGENTS.md": _document(3), "README.md": _document(3)})
    assert asked["reads"] == [("repo", "AGENTS.md")]


def test_no_base_means_the_diff_is_never_read():
    _, asked = _run({"AGENTS.md": _document(3)})
    assert asked["diffs"] == []


def test_a_base_is_diffed_against_the_root_under_test():
    _, asked = _run({"AGENTS.md": _document(3)}, base="origin/main", changed=["AGENTS.md"])
    assert asked["diffs"] == [("repo", "origin/main")]


def test_a_base_drops_a_closure_the_diff_never_reaches(capsys):
    code, _ = _run(
        {"AGENTS.md": _document(LIMITS.max_lines + 1)}, base="main", changed=["README.md"]
    )
    out = capsys.readouterr().out
    assert code == 0
    assert "::error" not in out
    assert "every instructions file fits its budget" in out


def test_a_base_keeps_a_closure_the_diff_reaches(capsys):
    code, _ = _run(
        {"AGENTS.md": _document(LIMITS.max_lines + 1)}, base="main", changed=["AGENTS.md"]
    )
    assert code == 1
    assert "::error file=AGENTS.md::" in capsys.readouterr().out


def test_a_base_keeps_an_importer_when_only_its_import_changed(capsys):
    code, _ = _run(
        {"AGENTS.md": "@a.md\n", "a.md": _document(LIMITS.max_lines + 1)},
        base="main",
        changed=["a.md"],
    )
    assert code == 1
    assert "::error file=AGENTS.md::" in capsys.readouterr().out


def test_a_raised_budget_passes_a_file_the_default_would_fail(capsys):
    code, _ = _run(
        {"AGENTS.md": _document(LIMITS.max_lines + 1)},
        limits=SimpleNamespace(warn_lines=100, warn_bytes=10000, max_lines=200, max_bytes=20000),
    )
    assert code == 0
    assert "::" not in capsys.readouterr().out
