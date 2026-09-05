"""The plugin's entry point: the hook pytest loads, delegating to ``apply_defaults``.

What the decorator adds — pytest treating it as a wrapper — is covered by ``tests/``.
"""
from __future__ import annotations

import pytest

from .apply_defaults import apply_defaults


@pytest.hookimpl(wrapper=True)
def pytest_load_initial_conftests(early_config, parser, args):
    apply_defaults(early_config, args)
    return (yield)
