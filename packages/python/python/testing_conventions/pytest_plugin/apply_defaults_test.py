"""Unit tests for the default fill-in: the decision applied, skipped, and swallowed.

Patches live in fixtures, per our own lints.
"""
from types import SimpleNamespace
from unittest import mock

import pytest

import testing_conventions.pytest_plugin.apply_defaults as unit


@pytest.fixture
def mock_decide():
    with mock.patch.object(unit, "decide") as patched:
        yield patched


@pytest.fixture
def mock_install_omit_patch():
    with mock.patch.object(unit, "install_omit_patch") as patched:
        yield patched


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
    unit.apply_defaults(config, [])
    assert config.known_args_namespace.cov_branch is True
    assert config.known_args_namespace.cov_fail_under == unit.FAIL_UNDER
    mock_install_omit_patch.assert_called_once_with(True)


def test_apply_defaults_leaves_options_alone_when_decision_is_false(
    mock_decide, mock_install_omit_patch
):
    mock_decide.return_value = SimpleNamespace(branch=False, fail_under=False, omit=False)
    config = _early_config()
    unit.apply_defaults(config, [])
    assert config.known_args_namespace.cov_branch is None
    assert config.known_args_namespace.cov_fail_under is None
    mock_install_omit_patch.assert_called_once_with(False)


def test_apply_defaults_passes_the_invocation_context_to_the_decision(
    mock_decide, mock_install_omit_patch
):
    mock_decide.return_value = SimpleNamespace(branch=False, fail_under=False, omit=False)
    unit.apply_defaults(_early_config(), ["--cov=pkg"])
    mock_decide.assert_called_once_with(["--cov=pkg"], "/x", None, unit.user_set)


def test_apply_defaults_is_a_noop_without_a_cov_run(mock_decide):
    unit.apply_defaults(_early_config(cov_source=None), [])
    mock_decide.assert_not_called()  # decide is never reached


def test_apply_defaults_swallows_errors(mock_decide):
    mock_decide.side_effect = ValueError("nope")
    unit.apply_defaults(_early_config(), [])  # must not raise
    mock_decide.assert_called_once()  # the error path was actually entered
