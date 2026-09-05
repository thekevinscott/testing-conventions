"""The uv-provisioning-wired decision — repo-only.

The reusable workflow provisions Python one way — uv — so the decision holds two facts at once:
the file carries no second provisioning tool (`actions/setup-python` / `python -m pip install`),
and the python-arm step sequence is identical across the three suite-executing jobs
(`unit-coverage`, `coverage-changed`, `mutation`), so a provisioning change lands in all three or
none. Comment lines are dropped before comparison — the steps are the provisioning contract, the
prose around them is per-job context.
"""
from __future__ import annotations

from checks.utils.extract_job_block import extract_job_block
from checks.uv_provisioning_wired.python_steps import python_steps

# Each suite-executing job, paired with the job header that bounds its own YAML region (its
# successor in file order), for `extract_job_block`.
SUITE_JOBS = (
    ("unit-coverage", "coverage-changed"),
    ("coverage-changed", "mutation"),
    ("mutation", "integration-lint"),
)


def decide(text: str) -> bool:
    """True when the workflow provisions Python with uv alone, identically in all three
    suite-executing jobs."""
    if "python -m pip install" in text or "actions/setup-python" in text:
        return False
    first, *rest = [python_steps(extract_job_block(text, start, end)) for start, end in SUITE_JOBS]
    return bool(first) and all(arm == first for arm in rest)
