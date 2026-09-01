"""Tests for the version-conditional TOML loader."""
import importlib
import io
import sys
import types
from unittest import mock

import pytest

import tomllib

import testing_conventions.config.tomlcompat as tomlcompat

_MODULE = "testing_conventions.config.tomlcompat"


@pytest.fixture
def tomllib_absent():
    """Make ``import tomllib`` raise and supply a fake ``tomli``; patch.dict restores both on exit."""
    fake_tomli = types.ModuleType("tomli")
    fake_tomli.load = lambda handle: {"from": "tomli"}
    with mock.patch.dict(sys.modules, {"tomllib": None, "tomli": fake_tomli}):
        yield fake_tomli


def test_load_parses_toml_bytes_into_a_mapping():
    assert tomlcompat.load(io.BytesIO(b"[tool]\nname = 'ruff'\n")) == {"tool": {"name": "ruff"}}


def test_uses_stdlib_tomllib_when_available():
    assert tomlcompat.load is tomllib.load


def test_falls_back_to_tomli_when_tomllib_is_absent(tomllib_absent):
    sys.modules.pop(_MODULE, None)  # drop the cache so the import re-runs
    reloaded = importlib.import_module(_MODULE)
    assert reloaded.load is tomllib_absent.load
