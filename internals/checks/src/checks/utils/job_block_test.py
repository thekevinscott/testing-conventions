"""Colocated unit test for the shared job-block extractor (isolation — only the UUT imported).

Drives `extract_job_block` directly over crafted multi-job YAML: it must include the start
header's own lines, stop before the next job's header, run to the end when no next header follows,
and yield nothing when the start header is absent.
"""
from checks.utils.job_block import extract_job_block

TEXT = """\
jobs:
  build:
    run: before-marker
  target:
    run: inside-marker
  next:
    run: after-marker
"""


def test_includes_the_start_job_and_excludes_earlier_and_later_jobs():
    block = extract_job_block(TEXT, "target", "next")
    assert "target:" in block
    assert "inside-marker" in block
    assert "before-marker" not in block  # the earlier `build` job is skipped
    assert "next:" not in block  # the next header closes the region (excluded)
    assert "after-marker" not in block  # and everything past it stays out


def test_runs_to_the_end_when_the_end_header_never_appears():
    # No `absent:` header follows, so `inside` is never reset — the block extends to end of text.
    block = extract_job_block(TEXT, "target", "absent")
    assert "inside-marker" in block
    assert "next:" in block
    assert "after-marker" in block


def test_is_empty_when_the_start_header_is_absent():
    assert extract_job_block(TEXT, "missing", "next") == ""
