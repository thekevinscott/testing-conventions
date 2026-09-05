"""The cli-node-engine-wired check — see docs/internals/repo.md, "The CLI runs on its own engine"."""
from __future__ import annotations

import json
from pathlib import Path

import click

from checks.cli_node_engine_wired.cli_jobs import CLI_INVOCATION, cli_jobs
from checks.cli_node_engine_wired.engine_floor import engine_floor
from checks.cli_node_engine_wired.violations import violations
from checks.config import NODE_PACKAGE_MANIFEST, REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
@click.argument("manifest", default=NODE_PACKAGE_MANIFEST, type=click.Path())
def cli(workflow: str, manifest: str) -> None:
    floor = engine_floor(json.loads(Path(manifest).read_text())["engines"]["node"])
    text = Path(workflow).read_text()
    jobs = cli_jobs(text)
    if not jobs:
        raise CheckFailed(
            f"no job in `{workflow}` matches `{CLI_INVOCATION}`, so this check inspected nothing and "
            "would pass at any node pin — point `CLI_INVOCATION` at the invocation the workflow now uses"
        )
    problems = violations(text, floor)
    if problems:
        raise CheckFailed(
            "a job invokes the published CLI on a node below its own `engines.node` floor, so npm "
            f"resolves the bare name to an older release: {'; '.join(problems)}"
        )
    click.echo(f"all {len(jobs)} CLI-invoking jobs provision node {floor} or newer")
