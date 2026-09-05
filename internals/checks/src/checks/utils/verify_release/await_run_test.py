"""Colocated unit tests for the run-registration wait (isolation — injected `run`/`sleep`/`clock`)."""
import json

from checks.utils.verify_release.await_run import RUN_APPEAR_TIMEOUT_S, RUN_POLL_INTERVAL_S, await_run


class _Result:
    def __init__(self, stdout="", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def test_timing_constants_are_the_expected_seconds():
    # Pin the literals so a NumberReplacer mutant on either is killed (they're referenced, not
    # value-asserted, everywhere else).
    assert RUN_APPEAR_TIMEOUT_S == 120
    assert RUN_POLL_INTERVAL_S == 10


def test_await_run_returns_the_registered_run_id():
    def run(argv, **kwargs):
        row = {"databaseId": 42, "headSha": "sha", "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:01Z"}
        return _Result(stdout=json.dumps([row]))

    assert await_run("a.yml", "sha", "2026-07-08T10:00:00Z", run, sleep=lambda _s: None, clock=lambda: 0.0) == 42


def test_await_run_retries_until_the_run_registers():
    listings = iter([json.dumps([]), json.dumps([
        {"databaseId": 9, "headSha": "sha", "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:01Z"}])])
    sleeps = []

    def run(argv, **kwargs):
        return _Result(stdout=next(listings))

    got = await_run("a.yml", "sha", "2026-07-08T10:00:00Z", run, sleep=sleeps.append, clock=lambda: 0.0)
    assert got == 9
    assert sleeps == [RUN_POLL_INTERVAL_S]  # waited once, by the poll interval, between attempts


def test_await_run_times_out_when_the_deadline_is_reached():
    # clock=[0, 120]: deadline = 0 + 120 = 120; the second read is *exactly* the deadline, so it
    # times out under `>=` — a `>` mutant would treat 120 > 120 as false and loop on to find the
    # run, so asserting the timeout (with the run available on the next poll) distinguishes them.
    listings = iter([json.dumps([]), json.dumps([
        {"databaseId": 9, "headSha": "sha", "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:01Z"}])])
    clock = iter([0.0, float(RUN_APPEAR_TIMEOUT_S)])

    def run(argv, **kwargs):
        return _Result(stdout=next(listings))

    try:
        await_run("a.yml", "sha", "2026-07-08T10:00:00Z", run, sleep=lambda _s: None, clock=lambda: next(clock))
    except TimeoutError as error:
        assert "never registered" in str(error)
    else:
        raise AssertionError("reaching the deadline must time out, not poll on")


def test_await_run_times_out_when_the_clock_passes_the_deadline():
    # clock=[0, 200]: the deadline is 120, and 200 is strictly *past* it, so `>=` times out — an
    # `>=`->`==` mutant would see 200 != 120, keep polling, and find the run instead of timing out.
    listings = iter([json.dumps([]), json.dumps([
        {"databaseId": 9, "headSha": "sha", "event": "workflow_dispatch", "createdAt": "2026-07-08T10:00:01Z"}])])
    clock = iter([0.0, float(RUN_APPEAR_TIMEOUT_S) + 80.0])

    def run(argv, **kwargs):
        return _Result(stdout=next(listings))

    try:
        await_run("a.yml", "sha", "2026-07-08T10:00:00Z", run, sleep=lambda _s: None, clock=lambda: next(clock))
    except TimeoutError as error:
        assert "never registered" in str(error)
    else:
        raise AssertionError("passing the deadline must time out, not poll on")
