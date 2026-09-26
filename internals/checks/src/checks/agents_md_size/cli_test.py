"""Colocated unit tests for the agents-md-size command."""
from checks.agents_md_size.cli import cli
from checks.agents_md_size.gate import BUDGET


def _patch_run(monkeypatch, verdict):
    """Stand a recording fake in for the gate; return the list of calls it saw."""
    seen = []

    def run(root, *, budget):
        seen.append((root, budget))
        return verdict

    monkeypatch.setattr("checks.agents_md_size.cli.run", run)
    return seen


def _exit_code(root=".", budget=BUDGET):
    """Drive the callback and return the exit code it raised."""
    try:
        cli.callback(root=root, budget=budget)
    except SystemExit as exit_:
        return exit_.code
    raise AssertionError("the command must exit through SystemExit")


def _param(name):
    return next(param for param in cli.params if param.name == name)


def test_declares_the_root_argument():
    root = _param("root")
    assert root.required is True


def test_declares_the_budget_option_defaulting_to_the_documented_cap():
    budget = _param("budget")
    assert budget.default == BUDGET
    assert budget.type is not None
    assert budget.required is False


def test_the_budget_option_takes_an_integer():
    assert _param("budget").type.convert("512", None, None) == 512


def test_threads_the_root_into_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _exit_code(root="pkg")
    assert seen == [("pkg", BUDGET)]


def test_threads_an_overridden_budget_into_the_gate(monkeypatch):
    seen = _patch_run(monkeypatch, 0)
    _exit_code(root="pkg", budget=512)
    assert seen == [("pkg", 512)]


def test_exits_zero_when_the_gate_holds(monkeypatch):
    _patch_run(monkeypatch, 0)
    assert _exit_code() == 0


def test_exits_nonzero_when_the_gate_reports_a_violation(monkeypatch):
    _patch_run(monkeypatch, 1)
    assert _exit_code() == 1
