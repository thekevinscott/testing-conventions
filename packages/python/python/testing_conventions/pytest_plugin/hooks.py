"""The pytest plugin's wiring.

The decidable logic lives elsewhere and is unit-tested: config detection in
``config.detect``, the default decision in ``plan``. This module wires that into
pytest. The hook delegates to ``apply_defaults`` (a plain function), so the only
thing the decorator adds — pytest treating it as a wrapper — is what the
``tests/`` integration suite covers; everything else is unit-tested here.
"""
from __future__ import annotations

import pytest

from ..config.detect import user_set
from .plan import FAIL_UNDER, OMIT, decide


def apply_omit(config):
    """Append our omit patterns to a coverage config (run and report)."""
    config.run_omit = list(config.run_omit or []) + OMIT
    config.report_omit = list(config.report_omit or []) + OMIT


def install_omit_patch(omit, coverage_module=None):
    """Patch ``Coverage.__init__`` so the omit decision is applied at construction
    — the one moment before measurement starts. Idempotent; ``omit`` is captured
    in the patch. ``coverage_module`` is injected by tests; in production it's
    imported lazily (this runs only when pytest-cov, hence coverage, is active)."""
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


def apply_defaults(early_config, args):
    """The hook body as a plain function: when a --cov run is active, fill in the
    defaults the consumer didn't set. Never raises — a config default must not
    break the consumer's run."""
    try:
        options = early_config.known_args_namespace
        if not getattr(options, "cov_source", None):
            return
        defaults = decide(
            args,
            early_config.invocation_params.dir,
            getattr(options, "cov_config", None),
            user_set,
        )
        if defaults.branch:
            options.cov_branch = True
        if defaults.fail_under:
            options.cov_fail_under = FAIL_UNDER
        install_omit_patch(defaults.omit)
    except Exception:
        pass


@pytest.hookimpl(wrapper=True)
def pytest_load_initial_conftests(early_config, parser, args):
    apply_defaults(early_config, args)
    return (yield)
