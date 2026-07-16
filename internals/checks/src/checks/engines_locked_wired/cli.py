"""The engines-locked-wired check — repo-only (#437).

Backs the `tc-checks engines-locked-wired` subcommand: every workflow layers the CI Python engine
toolchain (`coverage`, `pytest`, `cosmic-ray`, `maturin`) onto uv tool environments from the
hash-pinned `.github/uv/engines.txt`, never with a bare, floating `--with <engine>`. The `decide`
module holds the fact; this command scans every workflow file and names the offenders, and confirms
the lock the workflows point at exists.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import CI_ENGINE_LOCK, WORKFLOWS_DIR
from checks.engines_locked_wired.decide import floating_engines
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflows_dir", default=WORKFLOWS_DIR, type=click.Path())
@click.option("--lock", "lock", default=CI_ENGINE_LOCK, type=click.Path())
def cli(workflows_dir: str, lock: str) -> None:
    if not Path(lock).is_file():
        raise CheckFailed(
            f"the CI engine lock {lock} is missing — the workflows layer the engine toolchain "
            "from it, so it must be committed (#437)"
        )
    offenders = {
        path.name: floating_engines(path.read_text())
        for path in sorted(Path(workflows_dir).glob("*.yml"))
        if floating_engines(path.read_text())
    }
    if offenders:
        detail = "; ".join(f"{name}: {', '.join(engines)}" for name, engines in offenders.items())
        raise CheckFailed(
            "a workflow layers a CI engine with a bare, floating `--with` instead of "
            f"`--with-requirements {lock}` — the mutable external reference the lock removed is "
            f"back ({detail}) (#437)"
        )
    click.echo(f"every workflow layers the CI engine toolchain from {lock}, none floats a `--with`")
