"""The engines-drift check — repo-only (#439).

Backs the `tc-checks engines-drift` subcommand: the scheduled drift workflow recompiles
`.github/uv/engines.in` at latest into a temp file and hands its path here; this command compares
that latest resolution to the committed lock and reds — without blocking any PR, since the job is
scheduled and not a required check — when they diverge, naming what moved so the pin can be
advanced deliberately.

A standalone, colocated-tested check rather than inline `run: |` bash: the parse-and-compare is
real logic and earns real tests, and it stays clear of the GitHub Actions `${{ }}` templating trap.
The recompile itself is a plain `uv pip compile --upgrade` the workflow runs — a tool invocation,
no logic — so no subprocess boundary lives here.
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import CI_ENGINE_LOCK
from checks.engines_drift.decide import drift
from checks.utils.check_failed import CheckFailed


@click.command()
@click.option("--latest", "latest", required=True, type=click.Path(exists=True),
              help="Path to the freshly `--upgrade`-compiled latest resolution to compare against.")
@click.option("--lock", "lock", default=CI_ENGINE_LOCK, type=click.Path())
def cli(latest: str, lock: str) -> None:
    drifted = drift(Path(lock).read_text(), Path(latest).read_text())
    if drifted:
        raise CheckFailed(
            "the CI engine toolchain has drifted past the committed lock — the ecosystem moved, so "
            "advance the pin with `uv pip compile --upgrade .github/uv/engines.in` when ready and "
            "re-test (#439):\n" + "\n".join(drifted)
        )
    click.echo("the committed engine lock still matches the latest resolution — no drift")
