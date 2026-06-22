"""Unit tests for the pytest plugin wiring (#218).

Each function is driven directly — the hook as a plain generator, the monkeypatch
with an injected fake ``Coverage`` — so nothing here needs a live pytest run.
"""
from types import SimpleNamespace

import testing_conventions.pytest_plugin.hooks as hooks


def test_apply_omit_appends_to_empty():
    config = SimpleNamespace(run_omit=None, report_omit=None)
    hooks.apply_omit(config)
    assert config.run_omit == hooks.OMIT
    assert config.report_omit == hooks.OMIT


def test_apply_omit_preserves_existing():
    config = SimpleNamespace(run_omit=["a"], report_omit=["b"])
    hooks.apply_omit(config)
    assert config.run_omit == ["a"] + hooks.OMIT
    assert config.report_omit == ["b"] + hooks.OMIT


def _fake_coverage_module():
    class FakeCoverage:
        def __init__(self):
            self.config = SimpleNamespace(run_omit=[], report_omit=[])

    return SimpleNamespace(Coverage=FakeCoverage)


def test_install_applies_omit_at_construction():
    module = _fake_coverage_module()
    hooks.install_omit_patch(True, module)
    covered = module.Coverage()
    assert covered.config.run_omit == hooks.OMIT
    assert covered.config.report_omit == hooks.OMIT


def test_install_does_not_apply_omit_when_disabled():
    module = _fake_coverage_module()
    hooks.install_omit_patch(False, module)
    covered = module.Coverage()
    assert covered.config.run_omit == []


def test_install_is_idempotent():
    module = _fake_coverage_module()
    hooks.install_omit_patch(True, module)
    hooks.install_omit_patch(True, module)  # second call must not wrap again
    covered = module.Coverage()
    assert covered.config.run_omit == hooks.OMIT  # applied exactly once


def test_install_imports_real_coverage_when_not_injected():
    # Exercises the lazy `import coverage` path; idempotent and harmless.
    assert hooks.install_omit_patch(False) is None


def _early_config(cov_source="pkg"):
    namespace = SimpleNamespace(
        cov_source=cov_source, cov_branch=None, cov_fail_under=None, cov_config=None
    )
    return SimpleNamespace(
        known_args_namespace=namespace, invocation_params=SimpleNamespace(dir="/x")
    )


def test_apply_defaults_sets_options_from_the_decision(monkeypatch):
    monkeypatch.setattr(
        hooks, "decide", lambda *a, **k: SimpleNamespace(branch=True, fail_under=True, omit=True)
    )
    seen = {}
    monkeypatch.setattr(hooks, "install_omit_patch", lambda omit: seen.update(omit=omit))
    config = _early_config()
    hooks.apply_defaults(config, [])
    assert config.known_args_namespace.cov_branch is True
    assert config.known_args_namespace.cov_fail_under == hooks.FAIL_UNDER
    assert seen["omit"] is True


def test_apply_defaults_leaves_options_alone_when_decision_is_false(monkeypatch):
    monkeypatch.setattr(
        hooks, "decide", lambda *a, **k: SimpleNamespace(branch=False, fail_under=False, omit=False)
    )
    monkeypatch.setattr(hooks, "install_omit_patch", lambda omit: None)
    config = _early_config()
    hooks.apply_defaults(config, [])
    assert config.known_args_namespace.cov_branch is None
    assert config.known_args_namespace.cov_fail_under is None


def test_apply_defaults_is_a_noop_without_a_cov_run(monkeypatch):
    reached = []
    monkeypatch.setattr(hooks, "decide", lambda *a, **k: reached.append(1))
    hooks.apply_defaults(_early_config(cov_source=None), [])
    assert reached == []  # decide is never reached


def test_apply_defaults_swallows_errors(monkeypatch):
    def boom(*a, **k):
        raise ValueError("nope")

    monkeypatch.setattr(hooks, "decide", boom)
    hooks.apply_defaults(_early_config(), [])  # must not raise


def test_hook_is_registered_as_a_pytest_wrapper():
    # The decorator's *effect* (pytest dispatching this before pytest-cov) only
    # shows up in a live run, but pluggy records the opts on the function, so the
    # registration itself is unit-checkable — killing "remove decorator" and
    # "wrapper=True -> False".
    assert hooks.pytest_load_initial_conftests.pytest_impl["wrapper"] is True


def test_hook_delegates_and_passes_the_outcome_through(monkeypatch):
    seen = {}
    monkeypatch.setattr(hooks, "apply_defaults", lambda config, args: seen.update(config=config, args=args))
    config = _early_config()
    generator = hooks.pytest_load_initial_conftests(config, None, ["x"])
    next(generator)  # run up to the yield → apply_defaults runs
    assert seen["config"] is config
    assert seen["args"] == ["x"]
    try:
        generator.send("outcome")
    except StopIteration as stop:
        assert stop.value == "outcome"
