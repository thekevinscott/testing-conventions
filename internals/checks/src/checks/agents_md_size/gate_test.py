"""Colocated unit tests for the agents-md-size gate and its command (isolation — injected reads).

Both reads are injected as hand-rolled fakes, so the gate is exercised without a repo, a
subprocess, or a filesystem. The fakes record what they were asked for, which pins that `run`
threads its own root through rather than reading some other one.
"""
import inspect

from checks.agents_md_size.gate import cli, run

MAX_CHARS = 100


def _document(chars):
    return "x" * chars


def _fakes(files, asked):
    """Injected reads over a dict of path -> text, recording what each was asked for."""

    def tracked_paths(root):
        asked["roots"].append(root)
        return list(files)

    def read_text(root, path):
        asked["reads"].append((root, path))
        return files.get(path)

    return tracked_paths, read_text


def _run(files, *, max_chars=MAX_CHARS):
    """Drive `run` over a dict of path -> text; return (exit code, what each fake was asked)."""
    asked = {"roots": [], "reads": []}
    tracked_paths, read_text = _fakes(files, asked)
    code = run("repo", max_chars, tracked_paths=tracked_paths, read_text=read_text)
    return code, asked


def _run_with_default_budget(files):
    """Drive `run` without a budget argument, so the shipped default is the one under test."""
    asked = {"roots": [], "reads": []}
    tracked_paths, read_text = _fakes(files, asked)
    return run("repo", tracked_paths=tracked_paths, read_text=read_text), asked


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
    }


def test_the_shipped_budget_is_thirty_thousand_characters(capsys):
    # The literal, asserted through the message rather than against the constant: a test that
    # reads `MAX_CHARS` back moves with it and pins nothing.
    code, _ = _run_with_default_budget({"AGENTS.md": _document(30001)})
    out = capsys.readouterr().out
    assert code == 1
    assert "30001 characters" in out
    assert "30000-character budget" in out


def test_a_file_just_inside_the_shipped_budget_passes(capsys):
    code, _ = _run_with_default_budget({"AGENTS.md": _document(30000)})
    assert code == 0
    assert "::" not in capsys.readouterr().out


def test_a_tree_with_no_instructions_file_passes(capsys):
    code, _ = _run({"README.md": _document(MAX_CHARS * 5)})
    assert code == 0
    assert "every instructions file fits its budget" in capsys.readouterr().out


def test_a_small_instructions_file_passes(capsys):
    code, _ = _run({"AGENTS.md": _document(3)})
    assert code == 0
    assert "every instructions file fits its budget" in capsys.readouterr().out


def test_a_file_at_exactly_the_budget_passes(capsys):
    code, _ = _run({"AGENTS.md": _document(MAX_CHARS)})
    assert code == 0
    assert "::" not in capsys.readouterr().out


def test_a_file_one_character_over_the_budget_fails(capsys):
    code, _ = _run({"AGENTS.md": _document(MAX_CHARS + 1)})
    out = capsys.readouterr().out
    assert code == 1
    assert "::error file=AGENTS.md::" in out
    assert f"{MAX_CHARS + 1} characters" in out
    assert f"{MAX_CHARS}-character budget" in out
    assert "Move sections into reference files or skills" in out


def test_an_imported_file_counts_toward_the_importer(capsys):
    code, _ = _run({"AGENTS.md": "@a.md\n", "a.md": _document(MAX_CHARS)})
    assert code == 1
    assert "::error file=AGENTS.md::" in capsys.readouterr().out


def test_an_over_budget_file_suppresses_the_everything_fits_message(capsys):
    _run({"AGENTS.md": _document(MAX_CHARS + 1)})
    assert "every instructions file fits its budget" not in capsys.readouterr().out


def test_a_passing_run_annotates_nothing(capsys):
    _run({"AGENTS.md": _document(3)})
    assert "::" not in capsys.readouterr().out


def test_every_over_budget_file_is_reported_not_just_the_first(capsys):
    code, _ = _run(
        {"AGENTS.md": _document(MAX_CHARS + 1), "pkg/CLAUDE.md": _document(MAX_CHARS + 1)}
    )
    out = capsys.readouterr().out
    assert code == 1
    assert "::error file=AGENTS.md::" in out
    assert "::error file=pkg/CLAUDE.md::" in out


def test_a_nested_claude_md_is_held_to_the_same_budget(capsys):
    code, _ = _run({"pkg/CLAUDE.md": _document(MAX_CHARS + 1)})
    assert code == 1
    assert "::error file=pkg/CLAUDE.md::" in capsys.readouterr().out


def test_the_tree_is_listed_for_the_root_under_test():
    _, asked = _run({"AGENTS.md": _document(3)})
    assert asked["roots"] == ["repo"]


def test_every_read_is_rooted_at_the_root_under_test():
    _, asked = _run({"AGENTS.md": "@a.md\n", "a.md": "aye\n"})
    assert asked["reads"] == [("repo", "AGENTS.md"), ("repo", "a.md")]


def test_a_file_that_is_not_an_instructions_file_is_never_read():
    _, asked = _run({"AGENTS.md": _document(3), "README.md": _document(3)})
    assert asked["reads"] == [("repo", "AGENTS.md")]


def test_entries_are_visited_in_path_order():
    _, asked = _run({"pkg/AGENTS.md": _document(3), "AGENTS.md": _document(3)})
    assert asked["reads"] == [("repo", "AGENTS.md"), ("repo", "pkg/AGENTS.md")]


def test_a_raised_budget_passes_a_file_the_default_would_fail(capsys):
    code, _ = _run({"AGENTS.md": _document(MAX_CHARS + 1)}, max_chars=MAX_CHARS * 2)
    assert code == 0
    assert "::" not in capsys.readouterr().out


def _parameters():
    return {parameter.name: parameter for parameter in cli.params}


def _invoke(monkeypatch, verdict, **overrides):
    """Drive the command's callback against a recording fake gate; return (exit code, calls)."""
    seen = []

    def fake_run(root, max_chars):
        seen.append((root, max_chars))
        return verdict

    monkeypatch.setattr("checks.agents_md_size.gate.run", fake_run)
    arguments = {parameter.name: parameter.default for parameter in cli.params}
    arguments["root"] = "."
    try:
        cli.callback(**{**arguments, **overrides})
    except SystemExit as exit_:
        return exit_.code, seen
    raise AssertionError("the command must exit through SystemExit")


def test_the_command_declares_the_root_argument():
    assert _parameters()["root"].required is True


def test_the_budget_option_parses_as_an_integer():
    assert _parameters()["max_chars"].type.name == "integer"


def test_the_budget_option_declares_its_flag():
    assert "--max-chars" in _parameters()["max_chars"].opts


def test_the_command_threads_the_root_into_the_gate(monkeypatch):
    _, seen = _invoke(monkeypatch, 0, root="pkg")
    assert seen[0][0] == "pkg"


def test_the_command_passes_the_declared_budget_to_the_gate(monkeypatch):
    _, seen = _invoke(monkeypatch, 0)
    assert seen[0][1] == _parameters()["max_chars"].default


def test_the_command_passes_an_overridden_budget_to_the_gate(monkeypatch):
    _, seen = _invoke(monkeypatch, 0, max_chars=7)
    assert seen[0][1] == 7


def test_the_command_exits_zero_when_the_gate_holds(monkeypatch):
    code, _ = _invoke(monkeypatch, 0)
    assert code == 0


def test_the_command_exits_nonzero_when_the_gate_reports_a_violation(monkeypatch):
    code, _ = _invoke(monkeypatch, 1)
    assert code == 1
