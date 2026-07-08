"""Colocated unit tests for verify_release.

Unit-level: the pure decisions — version selection, missing-path detection, dispatched-run
selection — exercised in isolation (no git, no gh, no mocks). The orchestration
(`dispatch_and_wait`) is covered by the integration suite with the boundary mocked, and the
git-backed operations end to end by the e2e suite against a real repo — both under `tests/`.
"""
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_release as v  # noqa: E402


def test_published_version_picks_the_numeric_max_not_the_lexical_one():
    # v0.0.9 vs v0.0.67: a lexical max would wrongly pick "9". Numeric (major, minor, patch) wins.
    tags = [
        "testing-conventions-npm-v0.0.9",
        "testing-conventions-npm-v0.0.67",
        "testing-conventions-npm-v0.0.8",
    ]
    assert v.published_version(tags) == "0.0.67"


def test_published_version_ignores_non_npm_tags():
    tags = ["testing-conventions-rust-v0.0.99", "testing-conventions-npm-v0.0.2"]
    assert v.published_version(tags) == "0.0.2"


def test_published_version_raises_when_no_npm_tag_is_present():
    # No published binary to pin to → fail closed rather than verify against nothing.
    with pytest.raises(ValueError):
        v.published_version(["testing-conventions-rust-v0.0.1", "v0"])


def test_missing_paths_reports_absent_action_targets_in_order():
    present = {".github/actions/detect/action.yml", "README.md"}
    # detect.py is gone from the archive — exactly the file-move/export-ignore regression.
    assert v.missing_paths(present) == ["internals/detect/src/detect.py"]


def test_missing_paths_is_empty_when_every_required_path_is_present():
    present = set(v.REQUIRED_ACTION_PATHS) | {"other"}
    assert v.missing_paths(present) == []


def test_select_dispatched_run_picks_the_newest_matching_run():
    runs = [
        {"databaseId": 1, "headSha": "abc", "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:00Z"},
        {"databaseId": 2, "headSha": "abc", "event": "workflow_dispatch", "createdAt": "2026-07-08T12:00:00Z"},
    ]
    assert v.select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_ignores_other_shas_events_and_earlier_runs():
    runs = [
        {"databaseId": 1, "headSha": "other", "event": "workflow_dispatch", "createdAt": "2026-07-08T12:00:00Z"},
        {"databaseId": 2, "headSha": "abc", "event": "push", "createdAt": "2026-07-08T12:00:00Z"},
        {"databaseId": 3, "headSha": "abc", "event": "workflow_dispatch", "createdAt": "2026-07-08T08:00:00Z"},
        {"databaseId": 4, "headSha": "abc", "event": "workflow_dispatch", "createdAt": "2026-07-08T11:00:00Z"},
    ]
    # Only #4 matches sha + dispatch + at/after `since`; #1 (sha), #2 (event), #3 (too early) are out.
    assert v.select_dispatched_run(runs, "abc", since="2026-07-08T10:00:00Z")["databaseId"] == 4


def test_select_dispatched_run_raises_when_none_registered_yet():
    with pytest.raises(LookupError):
        v.select_dispatched_run([], "abc", since="2026-07-08T10:00:00Z")


def test_now_iso_is_a_utc_iso8601_timestamp():
    import re

    # The `createdAt` format select_dispatched_run compares against — the string sort it relies on
    # is only correct because both sides are this fixed-width UTC form.
    assert re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", v.now_iso())
