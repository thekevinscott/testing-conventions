"""Colocated unit tests for the commit-bodies git read (isolation — injected runner).

The exact argv is pinned: `%B` raw bodies over the two-dot range, so the `skip-changelog:` bypass
is findable on any line of any commit in the PR.
"""
from checks.changelog_gate.commit_messages import commit_messages


class _Result:
    """The slice of `subprocess.CompletedProcess` the wrapper reads."""

    def __init__(self, stdout):
        self.stdout = stdout


def _runner_returning(stdout):
    """A fake `subprocess.run` that records the call it saw and returns fixed stdout."""
    seen = []

    def runner(argv, **kwargs):
        seen.append((argv, kwargs))
        return _Result(stdout)

    runner.seen = seen
    return runner


def test_commit_messages_returns_raw_bodies_over_the_two_dot_range():
    runner = _runner_returning("fix: x\n\nskip-changelog: y\n")
    assert commit_messages("base", "head", runner=runner) == "fix: x\n\nskip-changelog: y\n"
    (argv, kwargs) = runner.seen[0]
    assert argv == ["git", "log", "--format=%B", "base..head"]
    assert kwargs == {"capture_output": True, "text": True, "check": True}
