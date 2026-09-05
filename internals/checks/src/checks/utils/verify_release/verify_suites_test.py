"""Colocated unit tests for the suite dispatch (isolation — an injected `run` fake, injected time)."""
import json
import re

from checks.utils.verify_release.verify_suites import now_iso, verify_suites


class _Result:
    def __init__(self, stdout=b"", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def test_now_iso_is_a_utc_iso8601_timestamp():
    assert re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", now_iso())


def _suite_run(dispatched, conclusions):
    """A `run` fake for verify_suites: records calls, answers `gh run list`/`view` from the given
    per-workflow databaseIds (`dispatched`) and conclusions."""
    calls = []

    def run(argv, **kwargs):
        calls.append(argv)
        if argv[:3] == ["gh", "run", "list"]:
            workflow = argv[argv.index("--workflow") + 1]
            row = {"databaseId": dispatched[workflow], "headSha": "sha", "event": "workflow_dispatch",
                   "createdAt": "2026-07-08T10:00:01Z"}
            return _Result(stdout=json.dumps([row]))
        if argv[:3] == ["gh", "run", "view"]:
            run_id = int(argv[3])
            return _Result(stdout=json.dumps({"status": "completed", "conclusion": conclusions[run_id]}))
        return _Result()

    run.calls = calls
    return run


def test_verify_suites_creates_the_ref_dispatches_all_and_returns_conclusions():
    run = _suite_run(dispatched={"a.yml": 1, "b.yml": 2}, conclusions={1: "success", 2: "failure"})
    result = verify_suites("sha", "0.0.67", ["a.yml", "b.yml"], run=run, now=lambda: "2026-07-08T10:00:00Z")
    assert result == {"a.yml": "success", "b.yml": "failure"}
    assert ["git", "push", "origin", "sha:refs/tags/verify-release-sha"] in run.calls
    dispatched = [c for c in run.calls if c[:3] == ["gh", "workflow", "run"]]
    assert [c[3] for c in dispatched] == ["a.yml", "b.yml"]
    assert dispatched[0][4:6] == ["--ref", "verify-release-sha"]
    assert dispatched[0][6:8] == ["-f", "version=0.0.67"]
    assert ["git", "push", "origin", ":refs/tags/verify-release-sha"] in run.calls


def test_verify_suites_deletes_the_ref_even_when_a_dispatch_raises():
    deleted = []

    def run(argv, **kwargs):
        if argv[:3] == ["gh", "workflow", "run"]:
            raise RuntimeError("gh boom")
        if argv == ["git", "push", "origin", ":refs/tags/verify-release-sha"]:
            deleted.append(argv)
        return _Result()

    try:
        verify_suites("sha", "0.0.67", ["a.yml"], run=run, now=lambda: "2026-07-08T10:00:00Z")
    except RuntimeError:
        pass
    else:
        raise AssertionError("the dispatch failure must propagate")
    assert deleted  # finally cleanup ran despite the failure
