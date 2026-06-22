"""Unit tests for the pytest plugin wiring (#218).

Each function is driven directly — the hook as a plain generator, the coverage
patch with an injected fake ``Coverage`` — so nothing here needs a live pytest
run, nor a real ``coverage`` install (the lazy-import path is exercised with a
fake module in ``sys.modules``). Patches live in fixtures, per our own lints.
"""
import sys
from types import SimpleNamespace
from unittest import mock

import pytest

import testing_conventions.pytest_plugin.hooks as hooks


def _fake_coverage_module():
    class FakeCoverage:
        def __init__(self):
            self.config = SimpleNamespace(run_omit=[], report_omit=[])

    return SimpleNamespace(Coverage=FakeCoverage)


@pytest.fixture
def fake_coverage_in_sys_modules():
    """Inject a fake ``coverage`` so the lazy ``import coverage`` resolves without
    a real install, and hand the fake back so the test can inspect the patch."""
    fake = _fake_coverage_module()
    with mock.patch.dict(sys.modules, {"coverage": fake}):
        yield fake


@pytest.fixture
def mock_decide():
    with mock.patch.object(hooks, "decide") as patched:
        yield patched


@pytest.fixture
def mock_install_omit_patch():
    with mock.patch.object(hooks, "install_omit_patch") as patched:
        yield patched


@pytest.fixture
def mock_apply_defaults():
    with mock.patch.object(hooks, "apply_defaults") as patched:
        yield patched


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


def test_install_imports_real_coverage_when_not_injected(fake_coverage_in_sys_modules):
    # Exercises the lazy `import coverage` path without a real coverage install:
    # the fake in sys.modules is what the bare `import coverage` resolves to, and
    # asserting the omit landed proves that import branch ran and patched it.
    assert hooks.install_omit_patch(True) is None
    assert fake_coverage_in_sys_modules.Coverage().config.run_omit == hooks.OMIT


def _early_config(cov_source="pkg"):
    namespace = SimpleNamespace(
        cov_source=cov_source, cov_branch=None, cov_fail_under=None, cov_config=None
    )
    return SimpleNamespace(
        known_args_namespace=namespace, invocation_params=SimpleNamespace(dir="/x")
    )


def test_apply_defaults_sets_options_from_the_decision(mock_decide, mock_install_omit_patch):
    mock_decide.return_value = SimpleNamespace(branch=True, fail_under=True, omit=True)
    config = _early_config()
    hooks.apply_defaults(config, [])
    assert config.known_args_namespace.cov_branch is True
    assert config.known_args_namespace.cov_fail_under == hooks.FAIL_UNDER
    mock_install_omit_patch.assert_called_once_with(True)


def test_apply_defaults_leaves_options_alone_when_decision_is_false(
    mock_decide, mock_install_omit_patch
):
    mock_decide.return_value = SimpleNamespace(branch=False, fail_under=False, omit=False)
    config = _early_config()
    hooks.apply_defaults(config, [])
    assert config.known_args_namespace.cov_branch is None
    assert config.known_args_namespace.cov_fail_under is None
    mock_install_omit_patch.assert_called_once_with(False)


def test_apply_defaults_is_a_noop_without_a_cov_run(mock_decide):
    hooks.apply_defaults(_early_config(cov_source=None), [])
    mock_decide.assert_not_called()  # decide is never reached


def test_apply_defaults_swallows_errors(mock_decide):
    mock_decide.side_effect = ValueError("nope")
    hooks.apply_defaults(_early_config(), [])  # must not raise
    mock_decide.assert_called_once()  # the error path was actually entered


def test_hook_is_registered_as_a_pytest_wrapper():
    # The decorator's *effect* (pytest dispatching this before pytest-cov) only
    # shows up in a live run, but pluggy records the opts on the function, so the
    # registration itself is unit-checkable — killing "remove decorator" and
    # "wrapper=True -> False".
    assert hooks.pytest_load_initial_conftests.pytest_impl["wrapper"] is True


def test_hook_delegates_and_passes_the_outcome_through(mock_apply_defaults):
    config = _early_config()
    generator = hooks.pytest_load_initial_conftests(config, None, ["x"])
    next(generator)  # run up to the yield → apply_defaults runs
    mock_apply_defaults.assert_called_once_with(config, ["x"])
    with pytest.raises(StopIteration) as excinfo:
        generator.send("outcome")  # the outcome must pass straight through
    assert excinfo.value.value == "outcome"
