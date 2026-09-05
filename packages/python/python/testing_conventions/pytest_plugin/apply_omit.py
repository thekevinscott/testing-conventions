"""Adding our omit patterns to a coverage config."""
from __future__ import annotations

from .plan import OMIT


def apply_omit(config):
    """Append our omit patterns to a coverage config (run and report)."""
    config.run_omit = list(config.run_omit or []) + OMIT
    config.report_omit = list(config.report_omit or []) + OMIT
