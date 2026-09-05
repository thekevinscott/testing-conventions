"""Colocated unit tests for the changed-paths git read (isolation — injected runner).

The exact argv is pinned: three-dot diffs from the merge base, so paths the base branch changed
after this branch forked are not read as this PR's work.
"""
from checks.changelog_gate.changed_files import changed_files


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


def test_changed_files_diffs_from_the_merge_base():
    runner = _runner_returning("a.rs\nb.rs\n")
    assert changed_files("base", "head", runner=runner) == ["a.rs", "b.rs"]
    (argv, kwargs) = runner.seen[0]
    assert argv == ["git", "diff", "--name-only", "base...head"]
    assert kwargs == {"capture_output": True, "text": True, "check": True}


def test_changed_files_drops_blank_lines():
    assert changed_files("base", "head", runner=_runner_returning("a.rs\n\n")) == ["a.rs"]


def test_changed_files_is_empty_for_an_empty_diff():
    assert changed_files("base", "head", runner=_runner_returning("")) == []
