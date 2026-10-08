"""Colocated unit tests for the npm-visibility wait (isolation — injected `run`/`sleep`/`clock`)."""
from checks.utils.verify_release.await_version import (
    VERSION_POLL_INTERVAL_S,
    VERSION_VISIBLE_TIMEOUT_S,
    await_version,
)


class _Result:
    def __init__(self, returncode=0):
        self.returncode = returncode


def _probe(returncodes):
    """A `run` fake answering each probe with the next code, recording every call it is handed."""
    codes = iter(returncodes)
    calls = []
    keywords = []

    def run(argv, **kwargs):
        calls.append(argv)
        keywords.append(kwargs)
        return _Result(returncode=next(codes))

    run.calls = calls
    run.keywords = keywords
    return run


def test_timing_constants_are_the_expected_seconds():
    assert VERSION_VISIBLE_TIMEOUT_S == 900
    assert VERSION_POLL_INTERVAL_S == 15


def test_the_probe_installs_and_runs_the_pinned_version_the_way_a_check_job_does():
    run = _probe([0])
    await_version("0.0.146", run, sleep=lambda _s: None, clock=lambda: 0.0)
    assert run.calls == [[
        "npm", "exec", "--yes", "--prefer-online", "--", "testing-conventions@0.0.146", "--help",
    ]]
    # A miss is the ordinary case, so its npm 404 stays out of the promotion log.
    assert run.keywords == [{"capture_output": True}]


def test_an_installable_version_returns_without_waiting():
    sleeps = []
    run = _probe([0])
    assert await_version("0.0.146", run, sleep=sleeps.append, clock=lambda: 0.0) is None
    assert sleeps == []


def test_a_registry_miss_is_polled_through_until_the_version_installs():
    sleeps = []
    run = _probe([1, 1, 0])
    await_version("0.0.146", run, sleep=sleeps.append, clock=lambda: 0.0)
    assert len(run.calls) == 3
    assert sleeps == [VERSION_POLL_INTERVAL_S, VERSION_POLL_INTERVAL_S]


def test_reaching_the_deadline_times_out_rather_than_polling_on():
    # clock=[0, 900]: the deadline is 0 + 900, and the second read lands *exactly* on it, so `>=`
    # times out where a `>` mutant would poll on and find the version installable.
    clock = iter([0.0, float(VERSION_VISIBLE_TIMEOUT_S)])
    run = _probe([1, 0])
    try:
        await_version("0.0.146", run, sleep=lambda _s: None, clock=lambda: next(clock))
    except TimeoutError as error:
        assert str(error) == (
            "testing-conventions@0.0.146 was not installable from npm within "
            f"{VERSION_VISIBLE_TIMEOUT_S}s"
        )
    else:
        raise AssertionError("reaching the deadline must time out, not poll on")


def test_passing_the_deadline_times_out():
    # clock=[0, 980]: strictly past the deadline, so an `>=`->`==` mutant would keep polling.
    clock = iter([0.0, float(VERSION_VISIBLE_TIMEOUT_S) + 80.0])
    run = _probe([1, 0])
    try:
        await_version("0.0.146", run, sleep=lambda _s: None, clock=lambda: next(clock))
    except TimeoutError as error:
        assert str(error).endswith(f"within {VERSION_VISIBLE_TIMEOUT_S}s")
    else:
        raise AssertionError("passing the deadline must time out, not poll on")
