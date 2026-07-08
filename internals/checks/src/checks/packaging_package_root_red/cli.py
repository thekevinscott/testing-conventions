"""The packaging-package-root-red check — repo-only (#302 #309, #321, #328).

Backs the `tc-checks packaging-package-root-red` subcommand: the `packaging-package-root-red` job in
`.github/workflows/testing-conventions-selftest.yml` drives the hermetic-CLI (built-from-HEAD) `packaging` command over a per-package `dist/` tarball that ships a test file and asserts the non-zero exit that fails a consumer's build (#302 #309).

A standalone, colocated-tested check rather than an inline `run: |` bash block: inline workflow
bash is untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:`
text is templated before the shell sees it).

The workflow step runs the hardcoded `CHECKS` through the shared `run_checks` orchestrator; a
trailing command may be passed instead (a single red-path check — the seam the e2e suite drives
with a benign `true`/`false`).
"""
from __future__ import annotations

import click

from checks.config import HERMETIC_CLI
from checks.utils.run_checks import run_checks

CHECKS = [
    (
        [*HERMETIC_CLI, "packaging", ".github/selftest/packaging-package-root/red/dist/packaging-package-root-fixture-0.0.0.tgz", "--language", "typescript"],
        True,
        "red per-package dist tarball trips the packaging check",
    ),
]


@click.command()
@click.argument("command", nargs=-1)
def cli(command) -> None:
    run_checks(CHECKS, command)
