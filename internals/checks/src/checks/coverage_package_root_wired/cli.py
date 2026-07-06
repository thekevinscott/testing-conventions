"""The coverage-package-root-wired check — repo-only (#278, #321).

Backs the `tc-checks coverage-package-root-wired` subcommand: both coverage jobs of the reusable
workflow (`.github/workflows/testing-conventions.yml`) — `unit-coverage` (whole-tree) and
`coverage-changed` (changed-line) — must reference `needs.detect.outputs.package_root` so their TS
install, Python-env provisioning, and `build_command` step run at the derived package root, not the
checkout root. The `decide` module isolates each job's own YAML region — a reference in a
neighbouring job must not satisfy it — and confirms the wiring lives inside both.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.coverage_package_root_wired.decide import decide
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not decide(Path(workflow).read_text()):
        raise CheckFailed(
            "a coverage job (unit-coverage / coverage-changed) does not reference "
            "needs.detect.outputs.package_root — TS install / Python env / build_command still "
            "run at the checkout root, not the derived package root (#278)"
        )
    click.echo("unit-coverage and coverage-changed install/build at the derived package root")
