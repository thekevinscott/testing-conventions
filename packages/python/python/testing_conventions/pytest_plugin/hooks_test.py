"""Unit tests for the pytest hook, driven directly as a plain generator.

Patches live in fixtures, per our own lints.
"""
from types import SimpleNamespace
from unittest import mock

import pytest

import testing_conventions.pytest_plugin.hooks as hooks


@pytest.fixture
def mock_apply_defaults():
    with mock.patch.object(hooks, "apply_defaults") as patched:
        yield patched


def test_hook_is_registered_as_a_pytest_wrapper():
    # The decorator's dispatch order only shows up in a live run, but pluggy records the opts
    # on the function, so "remove decorator" and "wrapper=True -> False" are still killable.
    assert hooks.pytest_load_initial_conftests.pytest_impl["wrapper"] is True


def test_hook_delegates_and_passes_the_outcome_through(mock_apply_defaults):
    config = SimpleNamespace()
    generator = hooks.pytest_load_initial_conftests(config, None, ["x"])
    next(generator)  # run up to the yield → apply_defaults runs
    mock_apply_defaults.assert_called_once_with(config, ["x"])
    with pytest.raises(StopIteration) as excinfo:
        generator.send("outcome")  # the outcome must pass straight through
    assert excinfo.value.value == "outcome"
