"""Colocated unit tests for the agents-md-size command.

The shipped budget values are pinned by the budgets module's own tests and by the e2e run; here
the claim is only that each option is declared and reaches the gate.
"""
from checks.agents_md_size.cli import cli

OPTIONS = {
    "--warn-lines": "warn_lines",
    "--warn-bytes": "warn_bytes",
    "--max-lines": "max_lines",
    "--max-bytes": "max_bytes",
}


def _patch_run(monkeypatch, verdict):
    """Stand a recording fake in for the gate; return the list of calls it was given."""
    seen = []

    def run(root, limits, base):
        seen.append((root, limits, base))
        return verdict

    monkeypatch.setattr("checks.agents_md_size.cli.run", run)
    return seen


def _invoke(**overrides):
    """Drive the callback with the declared defaults, overridden per case."""
    arguments = {parameter.name: parameter.default for parameter in cli.params}
    arguments["root"] = "."
    try:
        cli.callback(**{**arguments, **overrides})
    except SystemExit as exit_:
        return exit_.code
    raise AssertionError("the command must exit through SystemExit")


def _parameters():
    return {parameter.name: parameter for parameter in cli.params}


def test_declares_the_root_argument():
    assert _parameters()["root"].required is True


def test_the_base_option_defaults_to_the_whole_tree():
    assert _parameters()["base"].default is None


def test_each_budget_option_defaults_to_a_budget_rather_than_nothing():
    parameters = _parameters()
    assert [parameters[name].default > 0 for name in OPTIONS.values()] == [True] * len(OPTIONS)


def test_each_budget_option_parses_as_an_integer():
    parameters = _parameters()
    assert [parameters[name].type.name for name in OPTIONS.values()] == ["integer"] * len(OPTIONS)


def test_each_budget_option_declares_its_flag():
    parameters = _parameters()
    assert [flag in parameters[name].opts for flag, name in OPTIONS.items()] == [True] * len(OPTIONS)


def test_threads_the_root_into_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _invoke(root="pkg")
    assert seen[0][0] == "pkg"


def test_threads_the_base_into_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _invoke(base="origin/main")
    assert seen[0][2] == "origin/main"


def test_passes_the_declared_budgets_to_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    parameters = _parameters()
    _invoke()
    assert [getattr(seen[0][1], name) for name in OPTIONS.values()] == [
        parameters[name].default for name in OPTIONS.values()
    ]


def test_passes_overridden_budgets_to_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _invoke(warn_lines=1, warn_bytes=2, max_lines=3, max_bytes=4)
    assert [getattr(seen[0][1], name) for name in OPTIONS.values()] == [1, 2, 3, 4]


def test_exits_zero_when_the_gate_holds(monkeypatch):
    _patch_run(monkeypatch, 0)
    assert _invoke() == 0


def test_exits_nonzero_when_the_gate_reports_a_violation(monkeypatch):
    _patch_run(monkeypatch, 1)
    assert _invoke() == 1
