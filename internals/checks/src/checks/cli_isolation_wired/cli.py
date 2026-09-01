"""The cli-isolation-wired check — see docs/internals/repo.md, "The CLI resolves outside the checkout"."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed

CLI_SPEC = '"testing-conventions${VERSION:+@$VERSION}"'
ISOLATED_LAUNCHER = 'npm --prefix "$RUNNER_TEMP/testing-conventions-cli" exec --yes --'


def invocations(text: str) -> list[tuple[int, str]]:
    """The `(line number, line)` pairs in workflow `text` that name the CLI package spec."""
    return [(number, line) for number, line in enumerate(text.splitlines(), start=1) if CLI_SPEC in line]


def violations(text: str) -> list[str]:
    """The CLI invocations in workflow `text` that resolve against the checkout instead of a temp prefix."""
    return [f"line {number}: {line.strip()}" for number, line in invocations(text) if ISOLATED_LAUNCHER not in line]


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    text = Path(workflow).read_text()
    found = invocations(text)
    if not found:
        raise CheckFailed(
            f"no line in `{workflow}` names `{CLI_SPEC}`, so this check inspected nothing and would "
            "pass whatever the launcher is — point `CLI_SPEC` at the spec the workflow now uses"
        )
    problems = violations(text)
    if problems:
        raise CheckFailed(
            "an invocation resolves the CLI from the checkout, where a consumer's devDependency copy "
            f"shadows the intended release: {'; '.join(problems)}"
        )
    click.echo(f"all {len(found)} CLI invocations resolve from a temp prefix")
