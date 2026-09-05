"""Decide whether both coverage jobs reference `package_root` inside their own YAML region, so a
neighbouring job's reference satisfies neither."""
from __future__ import annotations

from checks.utils.extract_job_block import extract_job_block


def decide(text: str) -> bool:
    """True when both coverage jobs' own blocks reference detect's derived `package_root`."""
    marker = "needs.detect.outputs.package_root"
    return marker in extract_job_block(text, "unit-coverage", "coverage-changed") and (
        marker in extract_job_block(text, "coverage-changed", "mutation")
    )
