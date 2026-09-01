"""The cli-node-engine-wired check — see docs/internals/repo.md, "The CLI runs on its own engine"."""
from __future__ import annotations

import json
import re
from pathlib import Path

import click

from checks.config import NODE_PACKAGE_MANIFEST, REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed
from checks.utils.job_block import iter_job_blocks

# The package spec, not the `npx` prefix: the launcher around it changes, the spec is the invariant.
CLI_INVOCATION = '"testing-conventions${VERSION:+@$VERSION}"'

# Anchored on `- uses:` because a step opening that way carries no `if:`: a gated step, which
# leaves the CLI on the runner's ambient node, must not satisfy the check.
SETUP_NODE = re.compile(
    r"^      - uses: actions/setup-node@[^\n]*\n(?:        [^\n]*\n)*?          node-version: (\d+)",
    re.M,
)


def cli_jobs(text: str) -> list[tuple[str, str]]:
    """The `(name, block)` jobs in workflow `text` that invoke the published CLI."""
    return [(name, block) for name, block in iter_job_blocks(text) if CLI_INVOCATION in block]


def violations(text: str, floor: int) -> list[str]:
    """The CLI-invoking jobs in workflow `text` that do not unconditionally provision node `floor`."""
    problems = []
    for name, block in cli_jobs(text):
        pinned = [int(major) for major in SETUP_NODE.findall(block)]
        if not pinned:
            problems.append(f"`{name}` invokes the CLI with no unconditional `setup-node` step")
        elif max(pinned) < floor:
            problems.append(f"`{name}` pins node {max(pinned)}, below the floor of {floor}")
    return problems


def engine_floor(requirement: str) -> int:
    """The major node version an `engines.node` `requirement` demands, e.g. 24 from `>=24`."""
    match = re.fullmatch(r">=\s*(\d+)(?:\.\d+)*", requirement.strip())
    if match is None:
        raise CheckFailed(f"`engines.node` is `{requirement}`, not the `>=<major>` floor this check reads")
    return int(match.group(1))


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
