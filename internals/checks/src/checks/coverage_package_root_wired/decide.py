"""The coverage-package-root-wired decision — repo-only (#278, #321).

Both coverage jobs — `unit-coverage` (whole-tree) and `coverage-changed` (changed-line) — must
reference detect's derived `package_root` inside their own YAML region, so a reference in a
neighbouring job can't satisfy either. The decision ANDs the two block checks; the shared
`extract_job_block` confines each search to one job.
"""
from __future__ import annotations

from checks.utils.job_block import extract_job_block


def decide(text: str) -> bool:
    """True when both coverage jobs' own blocks reference detect's derived `package_root`."""
    marker = "needs.detect.outputs.package_root"
    return marker in extract_job_block(text, "unit-coverage", "coverage-changed") and (
        marker in extract_job_block(text, "coverage-changed", "mutation")
    )
