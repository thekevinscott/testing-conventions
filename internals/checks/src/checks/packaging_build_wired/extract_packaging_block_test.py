"""Colocated unit tests for the packaging block extraction (isolation — pure, no I/O)."""
from checks.packaging_build_wired.extract_packaging_block import extract_packaging_block

WIRED = """\
  packaging:
    name: Packaging (no test files in the built artifact)
    needs: detect
    steps:
      - run: echo build and scan
"""


def test_extract_packaging_block_stops_before_the_next_job():
    text = WIRED + "\n  next-job:\n    name: After\n"
    block = extract_packaging_block(text)
    assert "packaging:" in block
    assert "next-job" not in block


def test_extract_packaging_block_runs_to_end_when_no_next_job_follows():
    assert extract_packaging_block(WIRED) == WIRED


def test_extract_packaging_block_is_empty_when_the_job_is_absent():
    assert extract_packaging_block("  other-job:\n    name: X\n") == ""
