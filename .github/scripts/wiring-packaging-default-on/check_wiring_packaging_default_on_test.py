"""Colocated unit tests for check_wiring_packaging_default_on.

Unit-level: the pure `find_missing_wiring` decision over crafted WIRED / UNWIRED workflow text,
in isolation (no file reads, no subprocess, no network). The file-reading `main` and the
`__main__` guard are covered by the e2e suite.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_wiring_packaging_default_on as m  # noqa: E402

WIRED = "  packaging:\n    if: needs.detect.outputs.packaging_dist == 'true'\n"
UNWIRED = "  packaging:\n    if: always()\n"


def test_wired_returns_none():
    assert m.find_missing_wiring(WIRED) is None


def test_unwired_returns_error_message():
    assert m.find_missing_wiring(UNWIRED) == (
        "the packaging job doesn't gate on detect's `packaging_dist` — "
        "packaging isn't default-on (#186)"
    )


def test_empty_text_is_unwired():
    assert m.find_missing_wiring("") is not None
