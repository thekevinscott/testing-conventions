"""Colocated unit tests for verify_release — the validated-promotion orchestration (isolation).

The one external (git + `gh`) is the injected `run`; time is the injected `sleep`/`clock`/`now`.
So every parse and decision runs against fakes with no real subprocess and no patching — the
`stage_hermetic_cli` pattern (#356). Pure decisions are driven directly; the operations drive a
`run` fake that dispatches on argv and records calls.
"""
import json
import re

from checks.utils import verify_release as v


class _Result:
    def __init__(self, stdout=b"", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


# --- pure decisions ---

def test_published_version_picks_the_numeric_max_not_the_lexical_one():
    tags = [
        "testing-conventions-npm-v0.0.9",
        "testing-conventions-npm-v0.0.67",
        "testing-conventions-npm-v0.0.8",
    ]
    assert v.published_version(tags) == "0.0.67"


def test_published_version_ignores_non_npm_tags():
    assert v.published_version(["testing-conventions-rust-v0.0.99", "testing-conventions-npm-v0.0.2"]) == "0.0.2"


def test_published_version_raises_when_no_npm_tag_is_present():
    try:
        v.published_version(["testing-conventions-rust-v0.0.1", "v0"])
    except ValueError as error:
        assert "refusing to promote" in str(error)
    else:
        raise AssertionError("no npm tag must raise")


def test_missing_paths_reports_absent_targets_in_required_order():
    # Both absent → returned in REQUIRED_ACTION_PATHS order, not set order.
    assert v.missing_paths(set()) == list(v.REQUIRED_ACTION_PATHS)


def test_missing_paths_reports_only_the_absent_one():
    present = {v.REQUIRED_ACTION_PATHS[0], "unrelated"}
    assert v.missing_paths(present) == [v.REQUIRED_ACTION_PATHS[1]]


def test_missing_paths_is_empty_when_every_required_path_is_present():
    assert v.missing_paths(set(v.REQUIRED_ACTION_PATHS)) == []


def _run_row(sha="abc", event="workflow_dispatch", created="2026-07-08T10:00:00Z", db=7):
    return {"databaseId": db, "headSha": sha, "event": event, "createdAt": created}


def test_select_dispatched_run_picks_the_newest_matching_run():
    runs = [_run_row(created="2026-07-08T10:00:00Z", db=1), _run_row(created="2026-07-08T12:00:00Z", db=2)]
    assert v.select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_excludes_a_lexically_smaller_non_matching_sha():
    # sha "mmm"; a run at "aaa" (lexically < sha) must be excluded by `==` — a `<=` mutant would
    # wrongly include it. The wrong run is *newer*, so an `and`->`or` mutant (which would let the
    # matching event/time alone qualify it) would select it, and a `<=` mutant would too.
    runs = [_run_row(sha="aaa", created="2026-07-08T13:00:00Z", db=1), _run_row(sha="mmm", db=2)]
    assert v.select_dispatched_run(runs, "mmm", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_matches_a_non_interned_equal_sha():
    # `==`, not `is`: the caller's sha and the run's headSha are distinct string objects. An `is`
    # mutant would fail to match equal-but-not-identical strings and find nothing.
    sha = "".join(["a", "b", "c", "d"]) * 10  # 40 chars, freshly built (not interned)
    runs = [{"databaseId": 5, "headSha": "abcd" * 10, "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:00Z"}]
    assert v.select_dispatched_run(runs, sha, since="2026-07-08T09:00:00Z")["databaseId"] == 5


def test_select_dispatched_run_excludes_a_non_dispatch_event():
    # The non-dispatch run is newer, so an `and`->`or` mutant (sha+time alone qualifying it) would
    # wrongly select it over the real dispatch.
    runs = [_run_row(event="push", created="2026-07-08T13:00:00Z", db=1), _run_row(event="workflow_dispatch", db=2)]
    assert v.select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_includes_a_run_created_exactly_at_since():
    # `>=`, not `>`: a run created at the exact `since` timestamp is this verification's own run and
    # must be included; a `>` mutant would drop it.
    runs = [_run_row(created="2026-07-08T09:00:00Z", db=3)]
    assert v.select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 3


def test_select_dispatched_run_excludes_a_run_created_before_since():
    runs = [_run_row(created="2026-07-08T08:00:00Z", db=1), _run_row(created="2026-07-08T10:00:00Z", db=2)]
    assert v.select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_raises_when_none_match_yet():
    try:
        v.select_dispatched_run([], "abc", since="2026-07-08T09:00:00Z")
    except LookupError:
        pass
    else:
        raise AssertionError("no matching run must raise LookupError")


def test_failed_suites_labels_non_success_conclusions():
    assert v.failed_suites({"a.yml": "success", "b.yml": "failure"}) == ["b.yml (failure)"]


def test_failed_suites_names_a_missing_conclusion_rather_than_dropping_it():
    # `conclusion or 'no conclusion'`, not `and`: a None conclusion (cancelled/timed-out run) is a
    # failure and must be named, with a readable placeholder.
    assert v.failed_suites({"a.yml": None}) == ["a.yml (no conclusion)"]


def test_failed_suites_is_empty_when_every_suite_succeeded():
    # A freshly-built (non-interned) "success" string: `!= "success"`, not `is not "success"` — an
    # identity mutant would treat the equal-but-not-identical value as a failure.
    success = "".join(["succ", "ess"])
    assert v.failed_suites({"a.yml": success, "b.yml": success}) == []


def test_now_iso_is_a_utc_iso8601_timestamp():
    assert re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", v.now_iso())


def test_timing_constants_are_the_expected_seconds():
    # Pin the literals so a NumberReplacer mutant on either is killed (they're referenced, not
    # value-asserted, everywhere else).
    assert v.RUN_APPEAR_TIMEOUT_S == 120
    assert v.RUN_POLL_INTERVAL_S == 10


# --- operations (git + gh through the injected `run`) ---

def test_run_text_raises_when_the_command_exits_nonzero():
    def run(argv, **kwargs):
        return _Result(returncode=2)

    try:
        v.resolve_version("sha", run=run)
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "exited 2" in error.message
    else:
        raise AssertionError("a failing git command must raise")


def test_run_raises_on_a_signal_death():
    # A negative return code (POSIX signal, e.g. OOM-killed git) is nonzero and must raise too.
    def run(argv, **kwargs):
        return _Result(returncode=-9)

    try:
        v.check_layout("sha", run=run)
    except Exception as error:  # noqa: BLE001
        assert "exited -9" in error.message
    else:
        raise AssertionError("a signal-killed command must raise")


def test_resolve_version_reads_the_npm_tags_merged_into_the_sha():
    calls = []

    def run(argv, **kwargs):
        calls.append(argv)
        return _Result(stdout="testing-conventions-npm-v0.0.9\ntesting-conventions-npm-v0.0.67\n")

    assert v.resolve_version("thesha", run=run) == "0.0.67"
    assert calls[0] == ["git", "tag", "--merged", "thesha", "--list", "testing-conventions-npm-v*"]


def test_check_layout_returns_no_missing_paths_when_the_archive_carries_them():
    listing = "\n".join(v.REQUIRED_ACTION_PATHS) + "\n"

    def run(argv, **kwargs):
        return _Result(stdout=b"tar-bytes") if argv[:2] == ["git", "archive"] else _Result(stdout=listing.encode())

    assert v.check_layout("thesha", run=run) == []


def test_check_layout_reports_a_target_missing_from_the_archive():
    # detect.py stripped from the fetched tree — the file-move/export-ignore regression.
    listing = v.REQUIRED_ACTION_PATHS[0] + "\n"

    def run(argv, **kwargs):
        return _Result(stdout=b"tar-bytes") if argv[:2] == ["git", "archive"] else _Result(stdout=listing.encode())

    assert v.check_layout("thesha", run=run) == [v.REQUIRED_ACTION_PATHS[1]]


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
    result = v.verify_suites("sha", "0.0.67", ["a.yml", "b.yml"], run=run, now=lambda: "2026-07-08T10:00:00Z")
    assert result == {"a.yml": "success", "b.yml": "failure"}
    # Temp tag created at the sha and dispatched at, before cleanup deletes it.
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
        v.verify_suites("sha", "0.0.67", ["a.yml"], run=run, now=lambda: "2026-07-08T10:00:00Z")
    except RuntimeError:
        pass
    else:
        raise AssertionError("the dispatch failure must propagate")
    assert deleted  # finally cleanup ran despite the failure


def test_await_run_returns_the_registered_run_id():
    def run(argv, **kwargs):
        row = {"databaseId": 42, "headSha": "sha", "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:01Z"}
        return _Result(stdout=json.dumps([row]))

    assert v._await_run("a.yml", "sha", "2026-07-08T10:00:00Z", run, sleep=lambda _s: None, clock=lambda: 0.0) == 42


def test_await_run_retries_until_the_run_registers():
    listings = iter([json.dumps([]), json.dumps([
        {"databaseId": 9, "headSha": "sha", "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:01Z"}])])
    sleeps = []

    def run(argv, **kwargs):
        return _Result(stdout=next(listings))

    got = v._await_run("a.yml", "sha", "2026-07-08T10:00:00Z", run, sleep=sleeps.append, clock=lambda: 0.0)
    assert got == 9
    assert sleeps == [v.RUN_POLL_INTERVAL_S]  # waited once, by the poll interval, between attempts


def test_await_run_times_out_when_the_deadline_is_reached():
    # clock=[0, 120]: deadline = 0 + 120 = 120; the second read is *exactly* the deadline, so it
    # times out under `>=` — a `>` mutant would treat 120 > 120 as false and loop on to find the
    # run, so asserting the timeout (with the run available on the next poll) distinguishes them.
    listings = iter([json.dumps([]), json.dumps([
        {"databaseId": 9, "headSha": "sha", "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:01Z"}])])
    clock = iter([0.0, float(v.RUN_APPEAR_TIMEOUT_S)])

    def run(argv, **kwargs):
        return _Result(stdout=next(listings))

    try:
        v._await_run("a.yml", "sha", "2026-07-08T10:00:00Z", run, sleep=lambda _s: None, clock=lambda: next(clock))
    except TimeoutError as error:
        assert "never registered" in str(error)
    else:
        raise AssertionError("reaching the deadline must time out, not poll on")


def test_watch_conclusion_returns_once_the_run_completes():
    def run(argv, **kwargs):
        return _Result(stdout=json.dumps({"status": "completed", "conclusion": "success"}))

    assert v._watch_conclusion(3, run, sleep=lambda _s: None) == "success"


def test_watch_conclusion_polls_until_completion():
    states = iter([
        json.dumps({"status": "in_progress", "conclusion": None}),
        json.dumps({"status": "completed", "conclusion": "failure"}),
    ])
    sleeps = []

    def run(argv, **kwargs):
        return _Result(stdout=next(states))

    assert v._watch_conclusion(3, run, sleep=sleeps.append) == "failure"
    assert sleeps == [v.RUN_POLL_INTERVAL_S]  # waited once between the in-progress and completed polls
