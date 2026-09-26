"""Colocated unit tests for the agents-md-size command.

The shipped budget value is pinned by the gate's own tests and by the e2e run; here the claim is
only that the option is declared and reaches the gate.
"""
from checks.agents_md_size.cli import cli


def _patch_run(monkeypatch, verdict):
    """Stand a recording fake in for the gate; return the list of calls it was given."""
    seen = []

    def run(root, max_chars):
        seen.append((root, max_chars))
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


def test_the_budget_option_defaults_to_a_budget_rather_than_nothing():
    assert _parameters()["max_chars"].default > 0


def test_the_budget_option_parses_as_an_integer():
    assert _parameters()["max_chars"].type.name == "integer"


def test_the_budget_option_declares_its_flag():
    assert "--max-chars" in _parameters()["max_chars"].opts


def test_threads_the_root_into_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _invoke(root="pkg")
    assert seen[0][0] == "pkg"


def test_passes_the_declared_budget_to_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _invoke()
    assert seen[0][1] == _parameters()["max_chars"].default


def test_passes_an_overridden_budget_to_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _invoke(max_chars=7)
    assert seen[0][1] == 7


def test_exits_zero_when_the_gate_holds(monkeypatch):
    _patch_run(monkeypatch, 0)
    assert _invoke() == 0


def test_exits_nonzero_when_the_gate_reports_a_violation(monkeypatch):
    _patch_run(monkeypatch, 1)
    assert _invoke() == 1
