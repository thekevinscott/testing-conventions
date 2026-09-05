"""Colocated unit tests for the PR-head-pin decision (isolation — pure, no I/O).

`pins_pr_head` is driven over fixtures pinning inside the job, without a pin, with the job absent,
with no closing job (the block runs to end-of-text), and with the pin in a *later* job —
block-scoping must reject that last one.
"""
from checks.e2e_verify_checks_out_pr_head.pins_pr_head import pins_pr_head

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
    assert pins_pr_head(NO_CLOSING_JOB) is True


def test_false_when_the_pin_sits_in_a_later_job():
    # Block-scoping: a pin in the following `packaging:` job must not satisfy the e2e-verify
    # check — the block is extracted first, so only the e2e-verify job's own lines count.
    assert pins_pr_head(PIN_IN_LATER_JOB) is False
