"""Tests for the version-conditional TOML loader.

The fallback branch is dead on whichever interpreter you run on, but it's still
testable: force ``import tomllib`` to fail and re-import the module.
"""
import importlib
import sys
import types
from unittest import mock

import pytest

import tomllib

import testing_conventions.config.tomlcompat as tomlcompat

_MODULE = "testing_conventions.config.tomlcompat"


@pytest.fixture
def tomllib_absent():
    """Make ``import tomllib`` raise and supply a fake ``tomli`` fallback. The
    ``tomllib: None`` entry forces the ImportError; patch.dict restores
    sys.modules — including the cached real module dropped by the test — on exit."""
    fake_tomli = types.ModuleType("tomli")
    fake_tomli.load = lambda handle: {"from": "tomli"}
    with mock.patch.dict(sys.modules, {"tomllib": None, "tomli": fake_tomli}):
        yield fake_tomli


def test_uses_stdlib_tomllib_when_available():
    assert tomlcompat.load is tomllib.load


def test_falls_back_to_tomli_when_tomllib_is_absent(tomllib_absent):
    sys.modules.pop(_MODULE, None)  # drop the cache so the import re-runs
    reloaded = importlib.import_module(_MODULE)
    assert reloaded.load is tomllib_absent.load
