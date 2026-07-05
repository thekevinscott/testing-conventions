"""Colocated unit tests for check_coverage_package_root_wired.

Unit-level: the pure block-extraction and wiring detection, exercised on crafted YAML strings
in isolation (no file reads, no subprocess). The file-read, `main`, and `__main__` guard are
covered by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_coverage_package_root_wired as m  # noqa: E402

WIRED = """\
jobs:
  unit-coverage:
    steps:
      - run: npm ci --prefix ${{ needs.detect.outputs.package_root }}
  coverage-changed:
    steps:
      - run: uv sync --directory ${{ needs.detect.outputs.package_root }}
  mutation:
    steps:
      - run: echo done
"""

UNWIRED_UNIT_COVERAGE = """\
jobs:
  unit-coverage:
    steps:
      - run: npm ci
  coverage-changed:
    steps:
      - run: uv sync --directory ${{ needs.detect.outputs.package_root }}
  mutation:
    steps:
      - run: echo done
"""

UNWIRED_COVERAGE_CHANGED = """\
jobs:
  unit-coverage:
    steps:
      - run: npm ci --prefix ${{ needs.detect.outputs.package_root }}
  coverage-changed:
    steps:
      - run: uv sync
  mutation:
    steps:
      - run: echo done
"""


def test_extract_job_block_exact_inclusive_range():
    text = "before\n  a:\n    ref\n  b:\n    other\n"
    # Inclusive of both boundary headers; nothing before `  a:` and nothing after `  b:`.
    assert m.extract_job_block(text, "a", "b") == "  a:\n    ref\n  b:"


def test_extract_job_block_includes_both_boundary_headers():
    block = m.extract_job_block(WIRED, "unit-coverage", "coverage-changed")
    assert block.startswith("  unit-coverage:")
    assert block.endswith("  coverage-changed:")
    assert "npm ci --prefix ${{ needs.detect.outputs.package_root }}" in block
    # A reference belonging to the *next* job must not leak into this block.
    assert "uv sync" not in block


def test_extract_job_block_stops_at_first_end_header():
    block = m.extract_job_block(WIRED, "coverage-changed", "mutation")
    assert block.startswith("  coverage-changed:")
    assert block.endswith("  mutation:")
    assert "uv sync" in block
    assert "echo done" not in block


def test_extract_job_block_runs_to_eof_when_end_header_absent():
    block = m.extract_job_block(WIRED, "mutation", "no-such-job")
    assert block.startswith("  mutation:")
    assert "echo done" in block


def test_extract_job_block_empty_when_start_header_absent():
    assert m.extract_job_block(WIRED, "no-such-job", "mutation") == ""


def test_find_missing_wiring_returns_none_when_both_jobs_wired():
    assert m.find_missing_wiring(WIRED) is None


def test_find_missing_wiring_flags_unit_coverage():
    msg = m.find_missing_wiring(UNWIRED_UNIT_COVERAGE)
    assert msg is not None
    assert "the unit-coverage job doesn't reference" in msg
    assert "#278" in msg


def test_find_missing_wiring_flags_coverage_changed():
    msg = m.find_missing_wiring(UNWIRED_COVERAGE_CHANGED)
    assert msg is not None
    assert "the coverage-changed job doesn't reference" in msg
    assert "#278" in msg
