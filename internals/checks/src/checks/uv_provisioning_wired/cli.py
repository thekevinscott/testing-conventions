"""The uv-provisioning-wired check — repo-only (#399).

Backs the `tc-checks uv-provisioning-wired` subcommand: the reusable workflow
(`.github/workflows/testing-conventions.yml`) provisions Python with uv alone — one provisioning
tool, with the python-arm steps identical across the three suite-executing jobs (`unit-coverage`,
`coverage-changed`, `mutation`). The `decide` module holds both facts: no
`actions/setup-python` / `python -m pip install` second path anywhere in the file, and the
python-arm step sequence byte-identical (comments aside) across the three jobs, so a provisioning
change lands in all three or none.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed
from checks.uv_provisioning_wired.decide import decide


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not decide(Path(workflow).read_text()):
        raise CheckFailed(
            "the reusable workflow does not provision Python with uv alone — an "
            "actions/setup-python + `python -m pip install` arm survives, or the python-arm "
            "provisioning steps drift across unit-coverage / coverage-changed / mutation (#399)"
        )
    click.echo("the suite-executing jobs provision Python with uv alone, identically")
