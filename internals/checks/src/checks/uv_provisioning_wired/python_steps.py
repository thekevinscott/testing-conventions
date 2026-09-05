"""Pick a job's python-gated provisioning steps out of its YAML block."""
from __future__ import annotations


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
