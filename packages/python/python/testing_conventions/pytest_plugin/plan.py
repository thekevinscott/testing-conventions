"""The coverage-default decision, separate from the pytest hook glue so it can be
unit-tested directly: given the consumer's CLI args and config, decide which of
our defaults to apply (and leave alone anything they set themselves).
"""
from __future__ import annotations

from dataclasses import dataclass

# The recommended floor, matching the `unit coverage` rule's Python default.
OMIT = ["*_test.py", "*/conftest.py", "conftest.py"]
FAIL_UNDER = 100.0


@dataclass
class Defaults:
    """Which of our defaults to apply to this run."""

    branch: bool  # turn branch coverage on
    fail_under: bool  # set fail_under = FAIL_UNDER
    omit: bool  # add the OMIT patterns


def decide(args, start, cov_config, user_set):
    """Return the defaults to apply. ``user_set(start, cov_config, key, table)``
    reports whether the consumer set a key themselves; ``args`` are pytest's raw
    command-line args (so an explicit ``--cov-branch`` / ``--cov-fail-under``
    also counts as the consumer's choice)."""
    argv = " ".join(str(arg) for arg in args)
    return Defaults(
        branch="--cov-branch" not in argv and not user_set(start, cov_config, "branch", "run"),
        fail_under=(
            "--cov-fail-under" not in argv
            and not user_set(start, cov_config, "fail_under", "report")
        ),
        omit=not (
            user_set(start, cov_config, "omit", "run")
            or user_set(start, cov_config, "omit", "report")
        ),
    )
