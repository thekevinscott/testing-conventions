"""Colocated unit tests for check_colocated_rust_wired.

Unit-level: the pure text inspection over crafted WIRED / UNWIRED constants (no I/O). `main` and
the `__main__` guard are covered by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_colocated_rust_wired as m  # noqa: E402

WIRED = "language: ${{ fromJSON(needs.detect.outputs.colocated_test_languages) }}\n"
UNWIRED = "language: ${{ fromJSON(needs.detect.outputs.languages) }}\n"


def test_wired_workflow_reports_no_missing_wiring():
    assert m.find_missing_wiring(WIRED) is None


def test_unwired_workflow_reports_the_missing_matrix():
    msg = m.find_missing_wiring(UNWIRED)
    assert msg is not None
    assert "colocated_test_languages" in msg
    assert "#274" in msg


def test_path_from_argv_uses_default_when_no_argument():
    assert m.path_from_argv(["prog"], "the-default") == "the-default"


def test_path_from_argv_prefers_the_explicit_argument():
    assert m.path_from_argv(["prog", "other.yml"], "the-default") == "other.yml"
