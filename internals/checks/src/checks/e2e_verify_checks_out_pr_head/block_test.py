"""Colocated unit tests for the e2e-verify-checks-out-pr-head decision (isolation — pure, no I/O).

`extract_block` and `pins_pr_head` are driven directly over crafted fixtures: one pinning inside
the job, one without, one where the job is absent, one where no closing job follows (the block
runs to end-of-text), and one where the pin sits in a *later* job (block-scoping must reject it) —
so every loop branch and the scoping guarantee are exercised. Only stdlib `re` and the unit under
test are imported.
"""
import re

from checks.e2e_verify_checks_out_pr_head.block import extract_block, pins_pr_head

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
UNPINNED = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
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
PIN_IN_LATER_JOB = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "  packaging:\n"
    "    steps:\n"
    "      - with:\n"
    "          ref: ${{ github.event.pull_request.head.sha }}\n"
)


def test_true_when_the_pin_is_inside_the_job():
    assert pins_pr_head(PINNED) is True


def test_false_when_the_job_has_no_pin():
    assert pins_pr_head(UNPINNED) is False


def test_false_when_the_job_is_absent():
    assert pins_pr_head(NO_JOB) is False


def test_false_on_empty_text():
    assert pins_pr_head("") is False


def test_true_when_the_block_runs_to_end_of_text():
    # No closing `packaging:` job — the block extends to end-of-text; the pin still counts.
    assert pins_pr_head(NO_CLOSING_JOB) is True


def test_false_when_the_pin_sits_in_a_later_job():
    # Block-scoping: a pin in the following `packaging:` job must not satisfy the e2e-verify
    # check — the block is extracted first, so only the e2e-verify job's own lines count.
    assert pins_pr_head(PIN_IN_LATER_JOB) is False


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
