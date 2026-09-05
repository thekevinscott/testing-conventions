"""Unit tests for the coverage patch, against a fake ``Coverage``.

The fake is injected directly, and through ``sys.modules`` for the lazy-import path.
Patches live in fixtures, per our own lints.
"""
import sys
from types import SimpleNamespace
from unittest import mock

import pytest

import testing_conventions.pytest_plugin.install_omit_patch as unit


def _fake_coverage_module():
    class FakeCoverage:
        def __init__(self):
            self.config = SimpleNamespace(run_omit=[], report_omit=[])

    return SimpleNamespace(Coverage=FakeCoverage)


@pytest.fixture
def mock_apply_omit():
    with mock.patch.object(unit, "apply_omit") as patched:
        yield patched


@pytest.fixture
def fake_coverage_in_sys_modules():
    """Inject a fake ``coverage`` for the lazy ``import coverage`` to resolve to, and
    hand it back so the test can inspect the patch."""
    fake = _fake_coverage_module()
    with mock.patch.dict(sys.modules, {"coverage": fake}):
        yield fake


def test_install_applies_omit_at_construction(mock_apply_omit):
    module = _fake_coverage_module()
    unit.install_omit_patch(True, module)
    covered = module.Coverage()
    mock_apply_omit.assert_called_once_with(covered.config)


def test_install_does_not_apply_omit_when_disabled(mock_apply_omit):
    module = _fake_coverage_module()
    unit.install_omit_patch(False, module)
    module.Coverage()
    mock_apply_omit.assert_not_called()


def test_install_is_idempotent(mock_apply_omit):
    module = _fake_coverage_module()
    unit.install_omit_patch(True, module)
    unit.install_omit_patch(True, module)  # second call must not wrap again
    module.Coverage()
    assert mock_apply_omit.call_count == 1


def test_the_original_init_still_runs(mock_apply_omit):
    module = _fake_coverage_module()
    unit.install_omit_patch(True, module)
    assert module.Coverage().config.run_omit == []


def test_install_imports_real_coverage_when_not_injected(
    mock_apply_omit, fake_coverage_in_sys_modules
):
    assert unit.install_omit_patch(True) is None
    covered = fake_coverage_in_sys_modules.Coverage()
    mock_apply_omit.assert_called_once_with(covered.config)
