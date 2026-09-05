"""Applying the omit decision at the one moment before coverage measurement starts."""
from __future__ import annotations

from .apply_omit import apply_omit


def install_omit_patch(omit, coverage_module=None):
    """Patch ``Coverage.__init__`` so the omit decision is applied at construction
    — the one moment before measurement starts. Idempotent; ``omit`` is captured
    in the patch. ``coverage_module`` is injected by tests, else imported lazily."""
    if coverage_module is None:
        import coverage as coverage_module
    cls = coverage_module.Coverage
    if getattr(cls, "_tc_patched", False):
        return
    original_init = cls.__init__

    def __init__(self, *args, **kwargs):
        original_init(self, *args, **kwargs)
        if omit:
            apply_omit(self.config)

    cls.__init__ = __init__
    cls._tc_patched = True
