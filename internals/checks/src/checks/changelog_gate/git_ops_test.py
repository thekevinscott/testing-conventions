"""Colocated unit tests for the changelog-gate git reads (isolation — injected runner).

The subprocess boundary is injected as `runner`, so a hand-rolled fake stands in for it. Each
test pins the exact argv, because the diff semantics live entirely in those flags: three-dot for
the merge-base range, `--diff-filter=A` for added-only, two-dot for the commit bodies.
"""
from checks.changelog_gate.git_ops import added_files, changed_files, commit_messages


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


def test_changed_files_diffs_from_the_merge_base():
    runner = _runner_returning("a.rs\nb.rs\n")
    assert changed_files("base", "head", runner=runner) == ["a.rs", "b.rs"]
    (argv, kwargs) = runner.seen[0]
    # Three-dot: the files this branch changed, not the ones main changed after it forked.
    assert argv == ["git", "diff", "--name-only", "base...head"]
    assert kwargs == {"capture_output": True, "text": True, "check": True}


def test_changed_files_drops_blank_lines():
    assert changed_files("base", "head", runner=_runner_returning("a.rs\n\n")) == ["a.rs"]


def test_changed_files_is_empty_for_an_empty_diff():
    assert changed_files("base", "head", runner=_runner_returning("")) == []


def test_added_files_filters_the_diff_to_additions():
    runner = _runner_returning("new.md\n")
    assert added_files("base", "head", runner=runner) == ["new.md"]
    (argv, kwargs) = runner.seen[0]
    assert argv == ["git", "diff", "--name-only", "--diff-filter=A", "base...head"]
    assert kwargs == {"capture_output": True, "text": True, "check": True}


def test_added_files_drops_blank_lines():
    assert added_files("base", "head", runner=_runner_returning("new.md\n\n")) == ["new.md"]


def test_commit_messages_returns_raw_bodies_over_the_two_dot_range():
    runner = _runner_returning("fix: x\n\nskip-changelog: y\n")
    assert commit_messages("base", "head", runner=runner) == "fix: x\n\nskip-changelog: y\n"
    (argv, kwargs) = runner.seen[0]
    # `%B` is the raw body: the skip line is found on any line, not only a formal trailer.
    assert argv == ["git", "log", "--format=%B", "base..head"]
    assert kwargs == {"capture_output": True, "text": True, "check": True}
