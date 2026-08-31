"""The changelog-gate check — repo-only (#496).

Backs the `tc-checks changelog-gate` subcommand, which `.github/workflows/changelog.yml` runs on
every PR: a package whose source changed must add a changelog fragment and a migration fragment
under its own `changelog.d/` / `migrations.d/`.

The PR's base and head SHAs arrive as CLI arguments rather than an environment side-channel, and
git runs in the working directory — the checkout the workflow already made.
"""
from __future__ import annotations

import click

from checks.changelog_gate.gate import run


@click.command()
@click.argument("base_sha")
@click.argument("head_sha")
def cli(base_sha: str, head_sha: str) -> None:
    raise SystemExit(run(base_sha, head_sha))
