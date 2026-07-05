"""Colocated unit tests for check_e2e_verify_checks_out_pr_head.

Unit-level: the pure block extraction and the block-scoped pin check over crafted constants (no
I/O). The scoping is load-bearing — a pin outside the `e2e-verify` job must not satisfy the
check — so a negative case places the pin in a *different* job. `main` and the `__main__` guard
are covered by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_e2e_verify_checks_out_pr_head as m  # noqa: E402

WIRED = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "        with:\n"
    "          ref: ${{ github.event.pull_request.head.sha || github.sha }}\n"
    "  packaging:\n"
    "    steps:\n"
    "      - run: echo pack\n"
)

# The pin lives in a *different* job, never inside `e2e-verify` — a whole-file grep would be
# fooled, the block-scoped check must not be.
PIN_IN_WRONG_JOB = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "  packaging:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "        with:\n"
    "          ref: ${{ github.event.pull_request.head.sha }}\n"
)


def test_extract_block_returns_only_the_e2e_verify_job():
    block = m.extract_block(WIRED, m.JOB_START, m.JOB_END)
    assert block.startswith("  e2e-verify:")
    assert block.endswith("  packaging:")
    assert "echo pack" not in block


def test_extract_block_is_empty_when_start_absent():
    assert m.extract_block("  packaging:\n    steps:\n", m.JOB_START, m.JOB_END) == ""


def test_extract_block_runs_to_end_when_end_absent():
    text = "  e2e-verify:\n    steps:\n      - run: go\n"
    assert m.extract_block(text, m.JOB_START, m.JOB_END) == text.rstrip("\n")


def test_wired_job_reports_no_missing_pin():
    assert m.find_missing_pr_head_pin(WIRED) is None


def test_pin_in_a_different_job_is_still_reported_missing():
    msg = m.find_missing_pr_head_pin(PIN_IN_WRONG_JOB)
    assert msg is not None
    assert "head.sha" in msg


def test_path_from_argv_uses_default_when_no_argument():
    assert m.path_from_argv(["prog"], "the-default") == "the-default"


def test_path_from_argv_prefers_the_explicit_argument():
    assert m.path_from_argv(["prog", "other.yml"], "the-default") == "other.yml"
