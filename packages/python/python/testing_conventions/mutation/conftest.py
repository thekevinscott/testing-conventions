"""Shared fakes for the mutation adapter's unit tests.

The adapter imports ``cosmic_ray`` lazily (inside the functions), so the engine need not be
installed to unit-test it. This fixture installs a fake ``cosmic_ray`` package tree in
``sys.modules`` for the duration of a test, and the lazy imports resolve to it — injecting the
dependency via ``sys.modules`` (never ``monkeypatch`` or an inline ``patch``, which the
isolation lint forbids), the same technique the version-conditional ``tomlcompat`` test uses.
"""
from __future__ import annotations

import contextlib
import sys
from types import ModuleType, SimpleNamespace
from unittest.mock import MagicMock

import pytest


@pytest.fixture
def cosmic_ray():
    """Yield the adapter's cosmic-ray seams as configurable mocks. ``db`` is the fake
    ``WorkDB`` a test configures (``db.results`` for the baseline, ``db.completed_work_items``
    for the session); the rest are the ``find_modules`` / ``init`` / ``execute`` / ... calls."""
    saved = {
        name: mod
        for name, mod in sys.modules.items()
        if name == "cosmic_ray" or name.startswith("cosmic_ray.")
    }
    for name in saved:
        del sys.modules[name]

    seams = SimpleNamespace(
        db=MagicMock(name="WorkDB"),
        deserialize_config=MagicMock(name="deserialize_config"),
        find_modules=MagicMock(name="find_modules", return_value=[]),
        filter_paths=MagicMock(name="filter_paths", side_effect=lambda mods, excl: list(mods)),
        init=MagicMock(name="init"),
        execute=MagicMock(name="execute"),
        WorkItem=MagicMock(name="WorkItem"),
    )

    @contextlib.contextmanager
    def use_db(path, mode=None):
        yield seams.db

    def module(name, **attrs):
        mod = ModuleType(name)
        for key, value in attrs.items():
            setattr(mod, key, value)
        sys.modules[name] = mod
        return mod

    root = module("cosmic_ray")
    root.commands = module("cosmic_ray.commands", init=seams.init, execute=seams.execute)
    root.modules = module(
        "cosmic_ray.modules", find_modules=seams.find_modules, filter_paths=seams.filter_paths
    )
    root.config = module("cosmic_ray.config", deserialize_config=seams.deserialize_config)
    root.work_db = module(
        "cosmic_ray.work_db", use_db=use_db, WorkDB=SimpleNamespace(Mode=SimpleNamespace(create=1))
    )
    root.work_item = module("cosmic_ray.work_item", WorkItem=seams.WorkItem)

    yield seams

    for name in [n for n in sys.modules if n == "cosmic_ray" or n.startswith("cosmic_ray.")]:
        del sys.modules[name]
    sys.modules.update(saved)
