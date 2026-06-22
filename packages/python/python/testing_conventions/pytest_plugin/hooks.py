"""The pytest plugin's irreducible glue.

Everything decidable lives elsewhere and is unit-tested: the coverage-config
detection in ``config.detect``, the default decision in ``plan``. What remains
here is what can't be exercised without a real pytest + coverage run — the
wrapper hook that must beat pytest-cov to setting its options, and the global
``coverage.Coverage`` monkeypatch that injects ``omit`` at construction (the one
moment before measurement starts). The ``tests/`` integration suite drives it
end to end; this file is the sole exempted surface.
"""
from __future__ import annotations

import pytest

from ..config.detect import user_set
from .plan import FAIL_UNDER, OMIT, decide

_state = {"omit": False}


def _install_omit_patch():
    import coverage  # only reached when pytest-cov (hence coverage) is active

    if getattr(coverage.Coverage, "_tc_patched", False):
        return
    original_init = coverage.Coverage.__init__

    def __init__(self, *args, **kwargs):
        original_init(self, *args, **kwargs)
        if _state["omit"]:
            config = self.config
            config.run_omit = list(config.run_omit or []) + OMIT
            config.report_omit = list(config.report_omit or []) + OMIT

    coverage.Coverage.__init__ = __init__
    coverage.Coverage._tc_patched = True


@pytest.hookimpl(wrapper=True)
def pytest_load_initial_conftests(early_config, parser, args):
    try:
        options = early_config.known_args_namespace
        if getattr(options, "cov_source", None):  # a --cov run is active
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
            _state["omit"] = defaults.omit
            _install_omit_patch()
    except Exception:  # a config default must never break the consumer's run
        pass
    return (yield)
