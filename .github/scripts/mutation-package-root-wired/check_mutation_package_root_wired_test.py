"""Colocated unit tests for check_mutation_package_root_wired.

Unit-level: the pure job-block extraction and wiring detection, exercised on crafted YAML
strings in isolation (no file reads, no subprocess). The file-read, `main`, and the `__main__`
guard are covered by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_mutation_package_root_wired as m  # noqa: E402

WIRED = """\
jobs:
  coverage-changed:
    steps:
      - run: uv sync
  mutation:
    steps:
      - run: npm ci --prefix ${{ needs.detect.outputs.package_root }}
  integration-lint:
    steps:
      - run: echo done
"""

# The reference sits only in a *neighbouring* job, outside the mutation block.
UNWIRED = """\
jobs:
  coverage-changed:
    steps:
      - run: uv sync --directory ${{ needs.detect.outputs.package_root }}
  mutation:
    steps:
      - run: npm ci
  integration-lint:
    steps:
      - run: echo ${{ needs.detect.outputs.package_root }}
"""


def test_extract_job_block_exact_start_inclusive_end_exclusive():
    text = "before\n  a:\n    ref\n  b:\n    other\n"
    # From `  a:` (included) up to `  b:` (excluded); nothing before, nothing at/after `  b:`.
    assert m.extract_job_block(text, "a", "b") == "  a:\n    ref"


def test_extract_job_block_includes_start_excludes_end_header():
    block = m.extract_job_block(WIRED, "mutation", "integration-lint")
    assert block.startswith("  mutation:")
    assert "npm ci --prefix ${{ needs.detect.outputs.package_root }}" in block
    assert "  integration-lint:" not in block
    assert "echo done" not in block
    # A preceding job's lines stay out of the block.
    assert "uv sync" not in block


def test_extract_job_block_runs_to_eof_when_end_header_absent():
    block = m.extract_job_block(WIRED, "mutation", "no-such-job")
    assert block.startswith("  mutation:")
    assert "  integration-lint:" in block
    assert "echo done" in block


def test_extract_job_block_empty_when_start_header_absent():
    assert m.extract_job_block(WIRED, "no-such-job", "integration-lint") == ""


def test_find_missing_wiring_returns_none_when_wired():
    assert m.find_missing_wiring(WIRED) is None


def test_find_missing_wiring_ignores_reference_in_neighbouring_job():
    msg = m.find_missing_wiring(UNWIRED)
    assert msg == m.ERROR
    assert "#279" in msg
