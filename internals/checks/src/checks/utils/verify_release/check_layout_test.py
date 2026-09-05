"""Colocated unit tests for the archive layout check (isolation — an injected `run` fake)."""
from checks.utils.verify_release.check_layout import REQUIRED_ACTION_PATHS, check_layout, missing_paths


class _Result:
    def __init__(self, stdout=b"", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def test_missing_paths_reports_absent_targets_in_required_order():
    # Both absent → returned in REQUIRED_ACTION_PATHS order, not set order.
    assert missing_paths(set()) == list(REQUIRED_ACTION_PATHS)


def test_missing_paths_reports_only_the_absent_one():
    present = {REQUIRED_ACTION_PATHS[0], "unrelated"}
    assert missing_paths(present) == [REQUIRED_ACTION_PATHS[1]]


def test_missing_paths_is_empty_when_every_required_path_is_present():
    assert missing_paths(set(REQUIRED_ACTION_PATHS)) == []


def test_check_layout_returns_no_missing_paths_when_the_archive_carries_them():
    listing = "\n".join(REQUIRED_ACTION_PATHS) + "\n"

    def run(argv, **kwargs):
        return _Result(stdout=b"tar-bytes") if argv[:2] == ["git", "archive"] else _Result(stdout=listing.encode())

    assert check_layout("thesha", run=run) == []


def test_check_layout_reports_a_target_missing_from_the_archive():
    # detect.py stripped from the fetched tree — the file-move/export-ignore regression.
    listing = REQUIRED_ACTION_PATHS[0] + "\n"

    def run(argv, **kwargs):
        return _Result(stdout=b"tar-bytes") if argv[:2] == ["git", "archive"] else _Result(stdout=listing.encode())

    assert check_layout("thesha", run=run) == [REQUIRED_ACTION_PATHS[1]]


def test_check_layout_pipes_the_archive_bytes_into_the_tar_listing():
    seen = []

    def run(argv, **kwargs):
        seen.append((argv, kwargs))
        return _Result(stdout=b"tar-bytes")

    check_layout("thesha", run=run)
    assert seen[0] == (["git", "archive", "--format=tar", "thesha"], {"capture_output": True})
    assert seen[1] == (["tar", "--list", "--file", "-"], {"capture_output": True, "input": b"tar-bytes"})
