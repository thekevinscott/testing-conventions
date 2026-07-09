"""The cargo-target-dir-wired check — repo-only (#410).

Backs the `tc-checks cargo-target-dir-wired` subcommand: all four Rust-build cache steps
(`unit-coverage`, `coverage-changed`, `mutation`, `packaging`) must key their `target/` cache path
on `needs.detect.outputs.cargo_target_dir` — the workspace-aware directory (#410), with the same
stale-`@v0` `package_root`-derived fallback every other new-output adoption carries — rather than
the bare `needs.detect.outputs.package_root }}/target` path. Cargo resolves the target directory at
the *workspace* root regardless of the invoking working directory, so a workspace member's cache
keyed on its own package root archives (and restores) a directory that never exists — the exact
miss that made dirsql's suite jobs rebuild the same wheel on every run.

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

_WIRED_PATH = (
    "${{ needs.detect.outputs.cargo_target_dir || "
    "format('{0}/target', needs.detect.outputs.package_root) }}"
)
_BARE_PATH = "${{ needs.detect.outputs.package_root }}/target"
_EXPECTED_OCCURRENCES = 4


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    text = Path(workflow).read_text()
    wired = text.count(_WIRED_PATH)
    bare_survives = _BARE_PATH in text
    if wired != _EXPECTED_OCCURRENCES or bare_survives:
        raise CheckFailed(
            "expected all four Rust-build cache steps (unit-coverage, coverage-changed, "
            f"mutation, packaging) to key on needs.detect.outputs.cargo_target_dir with the "
            f"stale-@v0 fallback — found {wired}/{_EXPECTED_OCCURRENCES} wired, bare "
            f"package_root path present: {bare_survives} (#410)"
        )
    click.echo("all four Rust-build cache steps key on the derived cargo_target_dir")
