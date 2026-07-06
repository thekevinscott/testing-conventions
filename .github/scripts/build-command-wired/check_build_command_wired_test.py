"""Colocated unit tests for check_build_command_wired.

Unit-level: the pure `find_missing_wiring` decision, exercised over crafted wired/unwired
strings in isolation — no file reads, no subprocess, no network. `main` and the file-read
are covered by the e2e suite under `tests/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_build_command_wired as c  # noqa: E402

WIRED = "  build:\n    run: ${{ needs.detect.outputs.build_command }}\n"
UNWIRED = "  build:\n    run: echo no build step here\n"


def test_wired_workflow_has_no_problem():
    assert c.find_missing_wiring(WIRED) is None


def test_unwired_workflow_returns_the_error():
    assert c.find_missing_wiring(UNWIRED) == c.ERROR


def test_error_names_the_issue_and_input():
    assert "build_command" in c.ERROR
    assert "#243/#289" in c.ERROR


def test_bare_input_name_alone_is_enough():
    assert c.find_missing_wiring("needs.detect.outputs.build_command") is None


def test_empty_text_is_unwired():
    assert c.find_missing_wiring("") == c.ERROR
