"""The uv-provisioning-wired decision — repo-only (#399).

The reusable workflow provisions Python one way — uv — so the decision holds two facts at once:
the file carries no second provisioning tool (`actions/setup-python` / `python -m pip install`),
and the python-arm step sequence is identical across the three suite-executing jobs
(`unit-coverage`, `coverage-changed`, `mutation`), so a provisioning change lands in all three or
none. Comment lines are dropped before comparison — the steps are the provisioning contract, the
prose around them is per-job context.
"""
from __future__ import annotations

from checks.utils.job_block import extract_job_block

# Each suite-executing job, paired with the job header that bounds its own YAML region (its
# successor in file order), for `extract_job_block`.
SUITE_JOBS = (
    ("unit-coverage", "coverage-changed"),
    ("coverage-changed", "mutation"),
    ("mutation", "integration-lint"),
)


def python_steps(job_block: str) -> str:
    """The job's python-arm provisioning: every step chunk (a line opening `      - ` through the
    line before the next one) that mentions `matrix.language == 'python'`, with comment and blank
    lines dropped."""
    steps: list[list[str]] = []
    for line in job_block.splitlines():
        if line.startswith("      - "):
            steps.append([])
        if steps:
            steps[-1].append(line)
    picked = [
        "\n".join(line for line in step if line.strip() and not line.lstrip().startswith("#"))
        for step in steps
        if any("matrix.language == 'python'" in line for line in step)
    ]
    return "\n".join(picked)


def decide(text: str) -> bool:
    """True when the workflow provisions Python with uv alone, identically in all three
    suite-executing jobs."""
    if "python -m pip install" in text or "actions/setup-python" in text:
        return False
    first, *rest = [python_steps(extract_job_block(text, start, end)) for start, end in SUITE_JOBS]
    return bool(first) and all(arm == first for arm in rest)
