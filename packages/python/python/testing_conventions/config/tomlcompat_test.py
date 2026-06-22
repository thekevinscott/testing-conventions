"""Tests for the version-conditional TOML loader.

The fallback branch is dead on whichever interpreter you run on, but it's still
testable: force ``import tomllib`` to fail and re-import the module.
"""
import importlib
import sys
import types

import tomllib

import testing_conventions.config.tomlcompat as tomlcompat

_MODULE = "testing_conventions.config.tomlcompat"


def test_uses_stdlib_tomllib_when_available():
    assert tomlcompat.load is tomllib.load


def test_falls_back_to_tomli_when_tomllib_is_absent(monkeypatch):
    fake_tomli = types.ModuleType("tomli")
    fake_tomli.load = lambda handle: {"from": "tomli"}
    monkeypatch.setitem(sys.modules, "tomllib", None)  # makes `import tomllib` raise
    monkeypatch.setitem(sys.modules, "tomli", fake_tomli)
    monkeypatch.delitem(sys.modules, _MODULE, raising=False)
    try:
        reloaded = importlib.import_module(_MODULE)
        assert reloaded.load is fake_tomli.load
    finally:
        # Re-import the real module so other tests see the stdlib loader again.
        sys.modules.pop(_MODULE, None)
        importlib.import_module(_MODULE)
