"""Colocated unit tests for check_packaging_package_root_wired.

Unit-level: the pure wiring detection, exercised on crafted YAML strings in isolation (no file
reads, no subprocess). The file-read, `main`, and `__main__` guard are covered by the e2e
suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_packaging_package_root_wired as m  # noqa: E402

WIRED = """\
jobs:
  packaging:
    steps:
      - run: npx testing-conventions packaging ${{ needs.detect.outputs.package_root }}
"""

UNWIRED = """\
jobs:
  packaging:
    steps:
      - run: npx testing-conventions packaging dist/
"""


def test_find_missing_wiring_returns_none_when_wired():
    assert m.find_missing_wiring(WIRED) is None


def test_find_missing_wiring_returns_error_when_unwired():
    msg = m.find_missing_wiring(UNWIRED)
    assert msg == m.ERROR
    assert "#280" in msg


def test_find_missing_wiring_returns_error_on_empty_text():
    assert m.find_missing_wiring("") == m.ERROR
