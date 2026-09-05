"""The changelog-gate orchestration — repo-only.

Reads the PR's diff and commit bodies, then reports every package that changed source without
adding the fragments that document it. The three git reads are injected, so the orchestration is
exercised without a repo.

This gate reports a finding per package rather than raising `CheckFailed`, because a PR can owe
fragments for several packages at once and each one is a separate annotation the author has to
act on. It returns an exit code; the command turns that into the process's.
"""
from __future__ import annotations

import click

from checks.changelog_gate.changed_files import changed_files as read_changed_files
from checks.changelog_gate.code_touched import code_touched
from checks.changelog_gate.commit_messages import commit_messages as read_commit_messages
from checks.changelog_gate.decide import changed_packages, has_skip_line
from checks.changelog_gate.git_ops import added_files as read_added_files
from checks.changelog_gate.malformed_fragments import malformed_fragments
from checks.changelog_gate.missing_kinds import missing_kinds


def run(
    base_sha: str,
    head_sha: str,
    *,
    changed_files=read_changed_files,
    added_files=read_added_files,
    commit_messages=read_commit_messages,
) -> int:
    """Exit code for the range: 0 when the gate holds, 1 with an annotation per finding."""
    if has_skip_line(commit_messages(base_sha, head_sha)):
        click.echo("skip-changelog line present; bypassing changelog enforcement.")
        return 0

    changed = changed_files(base_sha, head_sha)
    added = added_files(base_sha, head_sha)

    failed = False
    for path in malformed_fragments(changed):
        click.echo(
            f"::error file={path}::fragment filenames must match YYYY-MM-DD-<slug>.md — the UTC "
            f"merge date, then lowercase letters, digits and hyphens. See docs/internals/repo.md, "
            f"'CHANGELOG + MIGRATIONS'."
        )
        failed = True

    packages = [pkg for pkg in changed_packages(changed) if code_touched(changed, pkg)]
    if not packages:
        if not failed:
            click.echo("No package source changed; nothing to enforce.")
        return 1 if failed else 0

    for pkg in packages:
        for kind in missing_kinds(added, pkg):
            click.echo(
                f"::error::{pkg} has code changes but no {kind} fragment was added. Add "
                f"{pkg}/{kind}.d/YYYY-MM-DD-<slug>.md, or put a 'skip-changelog: <reason>' line "
                f"on any commit for a genuinely internal refactor. See docs/internals/repo.md, "
                f"'CHANGELOG + MIGRATIONS'."
            )
            failed = True

    if failed:
        return 1
    click.echo("changelog and migrations fragments present for every changed package.")
    return 0
