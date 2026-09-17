"""Colocated unit tests for the tracked-paths git read (isolation — injected runner)."""
from checks.path_length_gate.git_ops import tracked_paths


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


def test_tracked_paths_lists_git_ls_files_output():
    runner = _runner_returning("a.md\nb.md\n")
    assert tracked_paths("root", runner=runner) == ["a.md", "b.md"]
    (argv, kwargs) = runner.seen[0]
    assert argv == ["git", "ls-files"]
    assert kwargs == {"cwd": "root", "capture_output": True, "text": True, "check": True}


def test_tracked_paths_drops_blank_lines():
    assert tracked_paths("root", runner=_runner_returning("a.md\n\n")) == ["a.md"]


def test_tracked_paths_is_empty_for_an_empty_tree():
    assert tracked_paths("root", runner=_runner_returning("")) == []
