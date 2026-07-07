"""The e2e-verify-scope-wired check — repo-only (#294 #319 #333, #321).

Backs the `tc-checks e2e-verify-scope-wired` subcommand: the reusable workflow's e2e-verify job
must (a) pass `--scope` naming `inputs.path` (the caller's own scoping input), not rely on the
derived `package_root` (which can be a real parent of `path`); (b) pass `--base` naming
`inputs.base` under a `github.event_name == 'pull_request'` gate, so freshness is diff-relative
(`<base>..HEAD`) rather than history-absolute — the model that keeps the gate adoptable by a
squash-merging repo (#319); and (c) append detect's rendered `$EXTRA_SCOPE` / `$EXCLUDE`
extra-freshness-root arguments (#333), so a shared source tree beside the package joins the walk.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated). It tracks the workflow file regardless of what the published binary ships; runtime
behavior is covered by the Rust e2e tests (`e2e_verify.rs`, `e2e_verify_e2e.rs`).
"""
from __future__ import annotations

import re
from pathlib import Path
from typing import Optional

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed

_JOB_START = re.compile(r"^  e2e-verify:", re.MULTILINE)
_NEXT_JOB = re.compile(r"^  packaging:", re.MULTILINE)
_HAS_SCOPE_FLAG = re.compile(r"--scope")
_HAS_SCAN_PATH_FROM_INPUTS_PATH = re.compile(r"SCAN_PATH:\s*.*inputs\.path")
_HAS_BASE_FLAG = re.compile(r"--base")
_HAS_BASE_FROM_INPUTS_BASE = re.compile(r"BASE:\s*.*inputs\.base")
_HAS_PULL_REQUEST_GATE = re.compile(r"github\.event_name == 'pull_request'")
# #333: the run step appends the detect-rendered `$EXTRA_SCOPE` / `$EXCLUDE` (repeated
# `--extra-scope`/`--exclude` arguments) and their env is wired from detect's outputs.
_HAS_EXTRA_SCOPE_ARG = re.compile(r"\$EXTRA_SCOPE")
_HAS_EXTRA_SCOPE_FROM_DETECT = re.compile(r"EXTRA_SCOPE:\s*.*e2e_extra_scope")
_HAS_EXCLUDE_ARG = re.compile(r"\$EXCLUDE")
_HAS_EXCLUDE_FROM_DETECT = re.compile(r"EXCLUDE:\s*.*e2e_exclude")

_SCOPE_ERROR = (
    "the e2e-verify job doesn't pass --scope naming inputs.path — the freshness walk "
    "is scoped to the derived package_root instead, which can be broader than what the "
    "caller's own path input names (#294)"
)
_BASE_ERROR = (
    "the e2e-verify job doesn't pass --base naming inputs.base — the freshness walk "
    "is history-absolute instead of scoped to this branch's `<base>..HEAD` diff, which "
    "reds unrelated PRs on a squash-merging repo (#319)"
)
_GATE_ERROR = (
    "the e2e-verify job isn't gated to `github.event_name == 'pull_request'` — its "
    "--base diff needs a base ref, so it must ride the same pull-request gate as the "
    "other diff-scoped jobs (#319)"
)
_EXTRA_SCOPE_ERROR = (
    "the e2e-verify job doesn't append $EXTRA_SCOPE from detect's e2e_extra_scope output — a "
    "shared source tree beside the package (a native core bound into several bindings) can't "
    "join the freshness walk, so a core-only PR leaves the binding attestation falsely fresh "
    "(#333)"
)
_EXCLUDE_ERROR = (
    "the e2e-verify job doesn't append $EXCLUDE from detect's e2e_exclude output — a "
    "feature-gated subtree of an extra root (a core cli/ compiled out of the bindings) can't "
    "be carved back out, so a change only under it would falsely stale the attestation (#333)"
)


def extract_e2e_verify_block(workflow_text: str) -> str:
    """The `e2e-verify:` job's own YAML, up to (not including) the next top-level job."""
    start = _JOB_START.search(workflow_text)
    if not start:
        return ""
    end = _NEXT_JOB.search(workflow_text, start.end())
    return workflow_text[start.start() : end.start() if end else len(workflow_text)]


def find_missing_wiring(workflow_text: str) -> Optional[str]:
    """None if the e2e-verify job passes --scope naming inputs.path and --base naming
    inputs.base under a pull-request gate, and appends the detect-rendered $EXTRA_SCOPE /
    $EXCLUDE arguments from detect's outputs; else the first error message."""
    block = extract_e2e_verify_block(workflow_text)
    if not _HAS_SCOPE_FLAG.search(block) or not _HAS_SCAN_PATH_FROM_INPUTS_PATH.search(block):
        return _SCOPE_ERROR
    if not _HAS_BASE_FLAG.search(block) or not _HAS_BASE_FROM_INPUTS_BASE.search(block):
        return _BASE_ERROR
    if not _HAS_PULL_REQUEST_GATE.search(block):
        return _GATE_ERROR
    if not _HAS_EXTRA_SCOPE_ARG.search(block) or not _HAS_EXTRA_SCOPE_FROM_DETECT.search(block):
        return _EXTRA_SCOPE_ERROR
    if not _HAS_EXCLUDE_ARG.search(block) or not _HAS_EXCLUDE_FROM_DETECT.search(block):
        return _EXCLUDE_ERROR
    return None


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    problem = find_missing_wiring(Path(workflow).read_text())
    if problem is not None:
        raise CheckFailed(problem)
    click.echo(
        "e2e-verify scopes the freshness walk to inputs.path, diffs inputs.base on PRs, and "
        "appends detect's extra-scope/exclude roots"
    )
