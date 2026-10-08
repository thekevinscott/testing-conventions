"""Colocated unit tests for the published-manifest read (isolation — an injected `run` fake)."""
from checks.published_detect_contract.published_manifest import DETECT_MANIFEST, published_manifest


class _Result:
    def __init__(self, stdout="", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def _git(stdout="outputs:\n", returncode=0):
    calls = []

    def run(argv, **kwargs):
        calls.append((argv, kwargs))
        return _Result(stdout=stdout, returncode=returncode)

    run.calls = calls
    return run


def test_the_manifest_is_read_out_of_the_ref_rather_than_the_working_tree():
    run = _git(stdout="name: 'Detect languages'\n")
    assert published_manifest("v0", run) == "name: 'Detect languages'\n"
    assert run.calls[0][0] == ["git", "show", "v0:.github/actions/detect/action.yml"]


def test_the_manifest_path_is_the_one_a_consumer_resolves():
    assert DETECT_MANIFEST == ".github/actions/detect/action.yml"


def test_a_ref_git_cannot_resolve_raises_rather_than_reading_an_empty_manifest():
    # An absent ref must fail closed: an empty manifest declares no outputs, which would read as
    # "every consumed name is missing" and bury the real cause under the whole output list.
    try:
        published_manifest("v9", _git(returncode=128))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "exited 128" in error.message
    else:
        raise AssertionError("an unresolvable ref must raise")
