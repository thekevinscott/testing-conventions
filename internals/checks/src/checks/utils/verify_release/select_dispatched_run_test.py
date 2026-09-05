"""Colocated unit tests for the dispatched-run selection (isolation — pure run rows in/out)."""
from checks.utils.verify_release.select_dispatched_run import select_dispatched_run


def _run_row(sha="abc", event="workflow_dispatch", created="2026-07-08T10:00:00Z", db=7):
    return {"databaseId": db, "headSha": sha, "event": event, "createdAt": created}


def test_select_dispatched_run_picks_the_newest_matching_run():
    runs = [_run_row(created="2026-07-08T10:00:00Z", db=1), _run_row(created="2026-07-08T12:00:00Z", db=2)]
    assert select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_excludes_a_lexically_smaller_non_matching_sha():
    # sha "mmm"; a run at "aaa" (lexically < sha) must be excluded by `==` — a `<=` mutant would
    # wrongly include it. The wrong run is *newer*, so an `and`->`or` mutant (which would let the
    # matching event/time alone qualify it) would select it, and a `<=` mutant would too.
    runs = [_run_row(sha="aaa", created="2026-07-08T13:00:00Z", db=1), _run_row(sha="mmm", db=2)]
    assert select_dispatched_run(runs, "mmm", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_excludes_a_lexically_greater_non_matching_sha():
    # sha "mmm"; a run at "zzz" (lexically > sha) must be excluded by `==` — a `>=` mutant would
    # wrongly include it. The wrong run is newer, so a `>=` mutant would select it over the match.
    runs = [_run_row(sha="zzz", created="2026-07-08T13:00:00Z", db=1), _run_row(sha="mmm", db=2)]
    assert select_dispatched_run(runs, "mmm", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_excludes_a_lexically_greater_non_dispatch_event():
    # event "zzz" sorts after "workflow_dispatch", so a `==`->`>=` mutant would wrongly include it;
    # newer, so the mutant would select it over the real dispatch.
    runs = [_run_row(event="zzz", created="2026-07-08T13:00:00Z", db=1), _run_row(event="workflow_dispatch", db=2)]
    assert select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_matches_a_non_interned_equal_sha():
    # `==`, not `is`: the caller's sha and the run's headSha are distinct string objects. An `is`
    # mutant would fail to match equal-but-not-identical strings and find nothing.
    sha = "".join(["a", "b", "c", "d"]) * 10  # 40 chars, freshly built (not interned)
    runs = [{"databaseId": 5, "headSha": "abcd" * 10, "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:00Z"}]
    assert select_dispatched_run(runs, sha, since="2026-07-08T09:00:00Z")["databaseId"] == 5


def test_select_dispatched_run_excludes_a_non_dispatch_event():
    # The non-dispatch run is newer, so an `and`->`or` mutant (sha+time alone qualifying it) would
    # wrongly select it over the real dispatch.
    runs = [_run_row(event="push", created="2026-07-08T13:00:00Z", db=1), _run_row(event="workflow_dispatch", db=2)]
    assert select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_includes_a_run_created_exactly_at_since():
    # `>=`, not `>`: a run created at the exact `since` timestamp is this verification's own run and
    # must be included; a `>` mutant would drop it.
    runs = [_run_row(created="2026-07-08T09:00:00Z", db=3)]
    assert select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 3


def test_select_dispatched_run_excludes_a_run_created_before_since():
    runs = [_run_row(created="2026-07-08T08:00:00Z", db=1), _run_row(created="2026-07-08T10:00:00Z", db=2)]
    assert select_dispatched_run(runs, "abc", since="2026-07-08T09:00:00Z")["databaseId"] == 2


def test_select_dispatched_run_raises_when_none_match_yet():
    try:
        select_dispatched_run([], "abc", since="2026-07-08T09:00:00Z")
    except LookupError:
        pass
    else:
        raise AssertionError("no matching run must raise LookupError")
