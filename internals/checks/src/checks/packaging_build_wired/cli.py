"""The packaging-build-wired check — repo-only (#335, #321).

Backs the `tc-checks packaging-build-wired` subcommand: the reusable workflow's packaging job
must (a) gate itself on detect's `packaging_build` output, so a package whose manifest states a
build runs the gate even with no committed dist/ and no uploaded artifact; (b) provision the
toolchain for detect's `packaging_language`, so the build has cargo/uv/node to run under;
(c) run the derived `packaging_build` (`eval "$PACKAGING_BUILD"`, wired from detect's output)
before scanning, so the distribution exists to scan; and (d) scan `target/package` alongside
`dist`, so a `cargo package` crate is found, not only a `uv build` / `<pm> pack` dist.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated). It tracks the workflow file regardless of what the published binary ships; runtime
behavior is covered by the Rust packaging tests.
"""
from __future__ import annotations

import re
from pathlib import Path
from typing import Optional

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed

_JOB_START = re.compile(r"^  packaging:", re.MULTILINE)
_NEXT_JOB = re.compile(r"^  [a-z][a-z0-9-]*:\s*$", re.MULTILINE)
# The `if:` gate must run the job when detect derived a build, not only for a committed
# dist / uploaded artifact — matched by the job-level alternation that ORs `packaging_build`
# with `packaging_dist`, so a build-step's own `if: ... packaging_build != ''` doesn't count.
_HAS_BUILD_GATE = re.compile(
    r"packaging_build != '' \|\| needs\.detect\.outputs\.packaging_dist == 'true'"
)
# The toolchain provisioning steps must branch on detect's `packaging_language`.
_HAS_LANGUAGE_PROVISION = re.compile(r"needs\.detect\.outputs\.packaging_language")
# The build step must run the derived command, wired from detect's `packaging_build`.
_HAS_BUILD_EVAL = re.compile(r'eval "\$PACKAGING_BUILD"')
_HAS_BUILD_FROM_DETECT = re.compile(r"PACKAGING_BUILD:\s*.*needs\.detect\.outputs\.packaging_build")
# The scan must reach `target/package`, where `cargo package` writes the crate.
_HAS_CRATE_SCAN = re.compile(r"target/package")

_GATE_ERROR = (
    "the packaging job's `if:` doesn't run on detect's packaging_build output — a native "
    "package whose manifest states a build (uv build / <pm> pack / cargo package) but ships "
    "no committed dist/ and no uploaded artifact is skipped, so the gate never runs (#335)"
)
_PROVISION_ERROR = (
    "the packaging job doesn't provision the toolchain for detect's packaging_language — the "
    "derived build has no cargo/uv/node to run under, so it fails before producing a "
    "distribution to scan (#335)"
)
_BUILD_ERROR = (
    "the packaging job doesn't run the derived packaging_build (`eval \"$PACKAGING_BUILD\"` "
    "wired from detect's packaging_build output) before scanning — no distribution is built, "
    "so a package with no committed dist/ has nothing to scan (#335)"
)
_CRATE_SCAN_ERROR = (
    "the packaging job doesn't scan target/package — a `cargo package` crate is written there, "
    "not under dist/, so a built crate is never inspected for shipped test files (#335)"
)


def extract_packaging_block(workflow_text: str) -> str:
    """The `packaging:` job's own YAML, up to (not including) the next top-level job (or the end
    of the file, since packaging is currently the last job)."""
    start = _JOB_START.search(workflow_text)
    if not start:
        return ""
    end = _NEXT_JOB.search(workflow_text, start.end())
    return workflow_text[start.start() : end.start() if end else len(workflow_text)]


def find_missing_wiring(workflow_text: str) -> Optional[str]:
    """None if the packaging job gates on packaging_build, provisions packaging_language, runs
    the derived packaging_build, and scans target/package; else the first error message."""
    block = extract_packaging_block(workflow_text)
    if not _HAS_BUILD_GATE.search(block):
        return _GATE_ERROR
    if not _HAS_LANGUAGE_PROVISION.search(block):
        return _PROVISION_ERROR
    if not _HAS_BUILD_EVAL.search(block) or not _HAS_BUILD_FROM_DETECT.search(block):
        return _BUILD_ERROR
    if not _HAS_CRATE_SCAN.search(block):
        return _CRATE_SCAN_ERROR
    return None


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    problem = find_missing_wiring(Path(workflow).read_text())
    if problem is not None:
        raise CheckFailed(problem)
    click.echo(
        "packaging gates on packaging_build, provisions packaging_language, builds the "
        "distribution, and scans target/package"
    )
