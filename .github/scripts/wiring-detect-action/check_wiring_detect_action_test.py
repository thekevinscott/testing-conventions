"""Colocated unit tests for check_wiring_detect_action.

Unit-level: the pure `find_missing_wiring` decision over crafted WIRED / UNWIRED workflow text,
in isolation (no file reads, no subprocess, no network). The file-reading `main` and the
`__main__` guard are covered by the e2e suite.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_wiring_detect_action as m  # noqa: E402

WIRED = """
jobs:
  detect:
    steps:
      - uses: ./.github/actions/detect@v0
"""

UNWIRED = """
jobs:
  detect:
    steps:
      - name: inline detection
        run: echo scanning
"""


def test_wired_returns_none():
    assert m.find_missing_wiring(WIRED) is None


def test_wired_bare_action_path_returns_none():
    assert m.find_missing_wiring("      - uses: actions/detect@main\n") is None


def test_unwired_returns_error_message():
    assert m.find_missing_wiring(UNWIRED) == (
        "the reusable workflow doesn't use the detect action — detection still runs as "
        "inline bash, off the tested engine (#185)"
    )


def test_uses_detect_without_ref_pin_is_not_wired():
    # No `@<ref>` — the pattern requires a pinned ref, so this is still unwired.
    assert m.find_missing_wiring("      - uses: ./.github/actions/detect\n") is not None
