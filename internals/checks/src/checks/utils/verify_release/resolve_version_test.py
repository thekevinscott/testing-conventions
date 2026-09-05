"""Colocated unit tests for the version resolution (isolation — an injected `run` fake)."""
from checks.utils.verify_release.resolve_version import resolve_version


class _Result:
    def __init__(self, stdout="", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def test_resolve_version_reads_the_npm_tags_merged_into_the_sha():
    calls = []

    def run(argv, **kwargs):
        calls.append(argv)
        return _Result(stdout="testing-conventions-npm-v0.0.9\ntesting-conventions-npm-v0.0.67\n")

    assert resolve_version("thesha", run=run) == "0.0.67"
    assert calls[0] == ["git", "tag", "--merged", "thesha", "--list", "testing-conventions-npm-v*"]


def test_resolve_version_strips_blank_listing_lines_before_picking():
    def run(argv, **kwargs):
        return _Result(stdout="\ntesting-conventions-npm-v0.0.4\n  \n")

    assert resolve_version("thesha", run=run) == "0.0.4"
