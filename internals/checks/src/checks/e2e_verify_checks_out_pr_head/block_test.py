"""Colocated unit tests for the block extraction (isolation — pure, no I/O).

`extract_block` is driven directly over crafted fixtures covering every loop branch: a closed
block, an absent start, and a block that runs to end-of-text. Only stdlib `re` and the unit under
test are imported.
"""
import re

from checks.e2e_verify_checks_out_pr_head.block import extract_block

JOB_START = re.compile(r"^  e2e-verify:")
JOB_END = re.compile(r"^  packaging:")

PINNED = (
    "  detect:\n"
    "    outputs: x\n"
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "        with:\n"
    "          ref: ${{ github.event.pull_request.head.sha || github.sha }}\n"
    "  packaging:\n"
    "    name: pkg\n"
)
NO_JOB = "  packaging:\n    name: pkg\n"
NO_CLOSING_JOB = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - with:\n"
    "          ref: ${{ github.event.pull_request.head.sha }}\n"
)


def test_extract_block_stops_at_the_closing_job():
    block = extract_block(PINNED, JOB_START, JOB_END)
    assert "e2e-verify:" in block
    assert "packaging:" in block  # inclusive of the boundary line
    assert "outputs: x" not in block  # the preceding job is excluded
    assert "name: pkg" not in block  # nothing past the boundary line


def test_extract_block_is_empty_when_the_start_is_absent():
    assert extract_block(NO_JOB, JOB_START, JOB_END) == ""


def test_extract_block_runs_to_end_when_no_closing_job_follows():
    block = extract_block(NO_CLOSING_JOB, JOB_START, JOB_END)
    assert "github.event.pull_request.head.sha" in block
