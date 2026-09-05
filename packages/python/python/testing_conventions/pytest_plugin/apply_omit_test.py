"""Unit tests for the omit append: an empty config and one the consumer already filled."""
from types import SimpleNamespace

import testing_conventions.pytest_plugin.apply_omit as unit


def test_apply_omit_appends_to_empty():
    config = SimpleNamespace(run_omit=None, report_omit=None)
    unit.apply_omit(config)
    assert config.run_omit == unit.OMIT
    assert config.report_omit == unit.OMIT


def test_apply_omit_preserves_existing():
    config = SimpleNamespace(run_omit=["a"], report_omit=["b"])
    unit.apply_omit(config)
    assert config.run_omit == ["a"] + unit.OMIT
    assert config.report_omit == ["b"] + unit.OMIT
