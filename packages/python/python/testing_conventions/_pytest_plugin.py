"""The ``testing_conventions`` pytest plugin.

Shipped in the ``testing-conventions`` wheel and auto-loaded by pytest (via the
``pytest11`` entry point), so a ``pip install testing-conventions`` — which a
consumer already does for the CLI — also brings the project's recommended
coverage floor to a local ``pytest --cov`` run: **branch coverage on**,
``fail_under = 100``, and test files (``*_test.py`` / ``conftest.py``) omitted
from the denominator.

These are *defaults*, not overrides: for any of the three the consumer has set
themselves — on the command line, or in their coverage config (``.coveragerc`` /
``setup.cfg`` / ``tox.ini`` / ``pyproject.toml``) — their value wins (see
``_config.user_set``). The plugin only acts when a coverage run is active
(``--cov``); a plain ``pytest`` is untouched, and it never raises into the run.

Mechanism: pytest-cov builds and *starts* coverage inside its own
``pytest_load_initial_conftests``, so we wrap that hook to set its branch /
fail-under options first, and gap-fill ``omit`` on the ``Coverage`` object at
construction (the one moment before measurement starts).
"""
from __future__ import annotations

import pytest

from ._config import user_set

# The recommended floor, matching the `unit coverage` rule's Python default.
OMIT = ["*_test.py", "*/conftest.py", "conftest.py"]
FAIL_UNDER = 100.0
_state = {"apply_omit": False}


def _install_omit_patch():
    """Append our ``omit`` to each ``Coverage`` object's config at construction —
    the only point before measurement starts, since pytest-cov starts coverage
    with no hook in between. Idempotent and gated on ``_state['apply_omit']``."""
    import coverage  # safe: only reached when pytest-cov (hence coverage) is active

    if getattr(coverage.Coverage, "_tc_patched", False):
        return
    original_init = coverage.Coverage.__init__

    def __init__(self, *args, **kwargs):
        original_init(self, *args, **kwargs)
        if _state["apply_omit"]:
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
            start = early_config.invocation_params.dir
            cov_config = getattr(options, "cov_config", None)
            argv = " ".join(str(a) for a in args)
            if "--cov-branch" not in argv and not user_set(start, cov_config, "branch", "run"):
                options.cov_branch = True
            if "--cov-fail-under" not in argv and not user_set(
                start, cov_config, "fail_under", "report"
            ):
                options.cov_fail_under = FAIL_UNDER
            keep_omit = user_set(start, cov_config, "omit", "run") or user_set(
                start, cov_config, "omit", "report"
            )
            _state["apply_omit"] = not keep_omit
            _install_omit_patch()
    except Exception:  # pragma: no cover - a config default must never break the run
        pass
    return (yield)
