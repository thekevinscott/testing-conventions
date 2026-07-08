"""Integration tests for verify_release's orchestration: real logic, the git/gh boundary mocked.

Per the standard, an integration test runs first-party code for real and mocks the externals.
git and the `gh` CLI are the externals here, so the `boundary` fixture patches the boundary
functions and yields the mocks; each test configures their return values and asserts the
orchestration's behaviour. The patching lives in the fixture, never inline in a test body.
"""
import sys
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))  # .github/scripts/verify-release
import verify_release as v  # noqa: E402


@pytest.fixture
def boundary():
    """Patch the git/gh boundary and yield the mocks for per-test setup + assertions."""
    with patch.object(v, "dispatch") as dispatch, \
            patch.object(v, "list_runs") as list_runs, \
            patch.object(v, "watch_conclusion") as watch, \
            patch.object(v, "create_ref") as create_ref, \
            patch.object(v, "delete_ref") as delete_ref, \
            patch.object(v, "now_iso", return_value="2026-07-08T10:00:00Z"):
        yield SimpleNamespace(dispatch=dispatch, list_runs=list_runs, watch=watch,
                              create_ref=create_ref, delete_ref=delete_ref)


def _run(sha="abc", created="2026-07-08T10:00:01Z", db=7):
    return {"databaseId": db, "headSha": sha, "event": "workflow_dispatch", "conclusion": None,
            "status": "in_progress", "createdAt": created}


def test_verify_suites_creates_the_ref_dispatches_all_then_returns_conclusions(boundary):
    # Two workflows: distinct run ids by workflow, distinct conclusions.
    boundary.list_runs.side_effect = lambda wf, **kw: [_run(db=1 if wf == "a.yml" else 2)]
    boundary.watch.side_effect = lambda rid, **kw: {1: "success", 2: "failure"}[rid]
    result = v.verify_suites("abc", "0.0.67", ["a.yml", "b.yml"])
    assert result == {"a.yml": "success", "b.yml": "failure"}
    boundary.create_ref.assert_called_once_with("verify-release-abc", "abc")
    # Both dispatched at the temp tag before either was awaited (parallel), then cleaned up.
    assert [c.args[0] for c in boundary.dispatch.call_args_list] == ["a.yml", "b.yml"]
    assert boundary.dispatch.call_args_list[0].args[1] == "verify-release-abc"
    boundary.delete_ref.assert_called_once_with("verify-release-abc")


def test_verify_suites_deletes_the_ref_even_when_a_dispatch_raises(boundary):
    boundary.dispatch.side_effect = RuntimeError("gh boom")
    with pytest.raises(RuntimeError):
        v.verify_suites("abc", "0.0.67", ["a.yml"])
    boundary.delete_ref.assert_called_once_with("verify-release-abc")  # finally cleanup ran


def test_await_run_retries_until_the_run_registers(boundary):
    # First poll: the dispatched run hasn't appeared yet (empty list). Second poll: it's there.
    boundary.list_runs.side_effect = [[], [_run()]]
    sleeps = []
    assert v.await_run("selftest.yml", "abc", "2026-07-08T09:00:00Z", sleep=sleeps.append) == 7
    assert sleeps  # it waited between polls rather than giving up on the first empty list
    assert boundary.list_runs.call_count == 2


def test_await_run_times_out_if_the_run_never_registers(boundary):
    boundary.list_runs.return_value = []  # never appears
    # A no-op sleep that advances a fake monotonic clock past the deadline on its first call.
    ticks = iter([0.0, v._RUN_APPEAR_TIMEOUT_S + 1])
    with patch.object(v.time, "monotonic", side_effect=lambda: next(ticks)):
        with pytest.raises(TimeoutError):
            v.await_run("selftest.yml", "abc", "2026-07-08T09:00:00Z", sleep=lambda _s: None)


def test_main_dispatch_and_wait_exits_zero_when_all_suites_pass(boundary, capsys):
    boundary.list_runs.return_value = [_run()]
    boundary.watch.return_value = "success"
    assert v.main(["verify_release.py", "dispatch-and-wait", "abc", "0.0.67", "selftest.yml"]) == 0


def test_main_dispatch_and_wait_exits_nonzero_and_fails_closed_on_a_red_suite(boundary, capsys):
    boundary.list_runs.return_value = [_run()]
    boundary.watch.return_value = "failure"
    assert v.main(["verify_release.py", "dispatch-and-wait", "abc", "0.0.67", "selftest.yml", "dogfood.yml"]) == 1
    out = capsys.readouterr().out
    assert "refusing to promote" in out
    assert "selftest.yml" in out


def test_main_check_layout_exits_zero_when_the_action_paths_are_present(capsys):
    with patch.object(v, "archive_paths", return_value=set(v.REQUIRED_ACTION_PATHS)):
        assert v.main(["verify_release.py", "check-layout", "abc"]) == 0


def test_main_check_layout_exits_nonzero_when_a_path_is_missing(capsys):
    with patch.object(v, "archive_paths", return_value={".github/actions/detect/action.yml"}):
        assert v.main(["verify_release.py", "check-layout", "abc"]) == 1
    out = capsys.readouterr().out
    assert "internals/detect/src/detect.py" in out
    assert "refusing to promote" in out


def test_main_resolve_version_prints_the_pinned_version(capsys):
    with patch.object(v, "reachable_npm_tags", return_value=["testing-conventions-npm-v0.0.67"]):
        assert v.main(["verify_release.py", "resolve-version", "abc"]) == 0
    assert capsys.readouterr().out.strip() == "0.0.67"


def test_main_rejects_an_unknown_command(capsys):
    assert v.main(["verify_release.py", "frobnicate"]) == 2


# The gh boundary functions, exercised with an injected fake `run` (the subprocess seam) so their
# argv construction and output parsing are covered without a real GitHub.

def test_dispatch_builds_the_gh_workflow_run_command():
    seen = []

    def fake(argv, **kwargs):
        seen.append((argv, kwargs))
        return SimpleNamespace(returncode=0, stdout="")

    v.dispatch("selftest.yml", "abc", "0.0.67", run=fake)
    assert seen[0][0] == ["gh", "workflow", "run", "selftest.yml", "--ref", "abc", "-f", "version=0.0.67"]
    assert seen[0][1].get("check") is True


def test_list_runs_parses_the_gh_json_into_dicts():
    payload = '[{"databaseId": 1, "headSha": "abc", "event": "workflow_dispatch", "status": "completed", "conclusion": "success", "createdAt": "2026-07-08T10:00:00Z"}]'

    def fake(argv, **kwargs):
        return SimpleNamespace(stdout=payload)

    runs = v.list_runs("selftest.yml", run=fake)
    assert runs == [{"databaseId": 1, "headSha": "abc", "event": "workflow_dispatch",
                     "status": "completed", "conclusion": "success", "createdAt": "2026-07-08T10:00:00Z"}]


def test_create_ref_pushes_the_temp_tag_at_the_sha():
    seen = []

    def fake(argv, **kwargs):
        seen.append(argv)
        return SimpleNamespace(returncode=0)

    v.create_ref("verify-release-abc", "abc", run=fake)
    assert seen[0] == ["git", "push", "origin", "abc:refs/tags/verify-release-abc"]


def test_delete_ref_deletes_the_temp_tag():
    seen = []

    def fake(argv, **kwargs):
        seen.append(argv)
        return SimpleNamespace(returncode=0)

    v.delete_ref("verify-release-abc", run=fake)
    assert seen[0] == ["git", "push", "origin", ":refs/tags/verify-release-abc"]


def test_watch_conclusion_returns_the_conclusion_once_completed():
    def fake(argv, **kwargs):
        return SimpleNamespace(stdout='{"status": "completed", "conclusion": "success"}')

    assert v.watch_conclusion(5, run=fake) == "success"


def test_watch_conclusion_polls_until_the_run_completes():
    outs = iter([
        '{"status": "in_progress", "conclusion": null}',
        '{"status": "completed", "conclusion": "failure"}',
    ])

    def fake(argv, **kwargs):
        return SimpleNamespace(stdout=next(outs))

    with patch.object(v.time, "sleep") as sleep:
        assert v.watch_conclusion(5, run=fake) == "failure"
    sleep.assert_called_once()  # it waited between the in-progress poll and the completed one
