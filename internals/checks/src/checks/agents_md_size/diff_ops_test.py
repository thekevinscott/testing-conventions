"""Colocated unit tests for the changed-paths git read (isolation — injected runner)."""
from checks.agents_md_size.diff_ops import changed_paths


class _Result:
    """The slice of `subprocess.CompletedProcess` this wrapper reads."""

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


def test_changed_paths_lists_the_diff_against_the_merge_base():
    runner = _runner_returning("AGENTS.md\ndocs/AGENTS.md\n")
    assert changed_paths("root", "origin/main", runner=runner) == ["AGENTS.md", "docs/AGENTS.md"]
    (argv, kwargs) = runner.seen[0]
    assert argv == ["git", "diff", "--name-only", "origin/main...HEAD"]
    assert kwargs == {"cwd": "root", "capture_output": True, "text": True, "check": True}


def test_changed_paths_drops_blank_lines():
    assert changed_paths("root", "main", runner=_runner_returning("AGENTS.md\n\n")) == ["AGENTS.md"]


def test_changed_paths_is_empty_for_an_untouched_branch():
    assert changed_paths("root", "main", runner=_runner_returning("")) == []
