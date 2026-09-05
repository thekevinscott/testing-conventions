"""The hook body as a plain function, so the decorator is all that stays in the wiring."""
from __future__ import annotations

from ..config.user_set import user_set
from .install_omit_patch import install_omit_patch
from .plan import FAIL_UNDER, decide


def apply_defaults(early_config, args):
    """Fill in the coverage defaults the consumer didn't set, when a --cov run is
    active. Never raises — a config default must not break the consumer's run."""
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
