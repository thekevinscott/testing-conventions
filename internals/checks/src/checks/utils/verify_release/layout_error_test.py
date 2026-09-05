"""Colocated unit tests for the layout messages (isolation — pure strings in/out)."""
from checks.utils.verify_release.layout_error import layout_error, layout_ok


def test_layout_error_names_the_sha_and_the_absent_paths_and_fails_closed():
    message = layout_error("thesha", ["a/b.yml", "c/d.py"])
    assert "thesha" in message
    assert "a/b.yml, c/d.py" in message
    assert "refusing to promote" in message


def test_layout_ok_names_the_sha():
    assert layout_ok("thesha") == "detect action layout present in the archive of thesha"
