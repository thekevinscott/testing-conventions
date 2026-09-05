"""Colocated unit tests for the completion watch (isolation — injected `run`/`sleep`)."""
import json

from checks.utils.verify_release.watch_conclusion import RUN_POLL_INTERVAL_S, watch_conclusion


class _Result:
    def __init__(self, stdout="", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def test_watch_conclusion_returns_once_the_run_completes():
    calls = []

    def run(argv, **kwargs):
        calls.append(argv)
        return _Result(stdout=json.dumps({"status": "completed", "conclusion": "success"}))

    assert watch_conclusion(3, run, sleep=lambda _s: None) == "success"
    assert calls == [["gh", "run", "view", "3", "--json", "status,conclusion"]]


def test_watch_conclusion_keeps_polling_on_a_status_that_sorts_below_completed():
    # `== "completed"`, not `<= "completed"`: a status that sorts lexically *below* "completed"
    # (here a fabricated "aborted") is not terminal — a `<=` mutant would stop early and return its
    # conclusion instead of polling on to the real completed state.
    states = iter([
        json.dumps({"status": "aborted", "conclusion": "wrong"}),
        json.dumps({"status": "completed", "conclusion": "success"}),
    ])

    def run(argv, **kwargs):
        return _Result(stdout=next(states))

    assert watch_conclusion(3, run, sleep=lambda _s: None) == "success"


def test_watch_conclusion_polls_until_completion():
    states = iter([
        json.dumps({"status": "in_progress", "conclusion": None}),
        json.dumps({"status": "completed", "conclusion": "failure"}),
    ])
    sleeps = []

    def run(argv, **kwargs):
        return _Result(stdout=next(states))

    assert watch_conclusion(3, run, sleep=sleeps.append) == "failure"
    assert sleeps == [RUN_POLL_INTERVAL_S]  # waited once between the in-progress and completed polls


def test_watch_conclusion_reads_an_absent_conclusion_as_empty():
    def run(argv, **kwargs):
        return _Result(stdout=json.dumps({"status": "completed", "conclusion": None}))

    assert watch_conclusion(3, run, sleep=lambda _s: None) == ""
