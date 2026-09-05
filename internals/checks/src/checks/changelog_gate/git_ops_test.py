"""Colocated unit tests for the added-paths git read (isolation — injected runner).

The subprocess boundary is injected as `runner`, so a hand-rolled fake stands in for it. The exact
argv is pinned, because the diff semantics live entirely in those flags: three-dot for the
merge-base range, `--diff-filter=A` for added-only.
"""
from checks.changelog_gate.git_ops import added_files


class _Result:
    """The slice of `subprocess.CompletedProcess` these wrappers read."""

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


def test_added_files_filters_the_diff_to_additions():
    runner = _runner_returning("new.md\n")
    assert added_files("base", "head", runner=runner) == ["new.md"]
    (argv, kwargs) = runner.seen[0]
    assert argv == ["git", "diff", "--name-only", "--diff-filter=A", "base...head"]
    assert kwargs == {"capture_output": True, "text": True, "check": True}


def test_added_files_drops_blank_lines():
    assert added_files("base", "head", runner=_runner_returning("new.md\n\n")) == ["new.md"]


def test_added_files_is_empty_for_an_empty_diff():
    assert added_files("base", "head", runner=_runner_returning("")) == []
