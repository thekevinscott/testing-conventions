"""Colocated unit test for the coverage decision (isolation — only the UUT imported).

`decide` ANDs two block checks, so both a unit-coverage-only-missing and a
coverage-changed-only-missing fixture appear: each leaves the other operand true, forcing the
`and` to carry weight (dropping either condition, or flipping `and`→`or`, fails a distinct case).
The block extraction it calls is the shared `job_block` util, exercised through this real call
(not imported here — that would read as a collaborator).
"""
from checks.coverage_package_root_wired.decide import decide

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


def test_true_when_both_coverage_jobs_are_wired():
    assert decide(WIRED) is True


def test_false_when_only_unit_coverage_is_unwired():
    # coverage-changed is wired; the first operand is false, so the `and` short-circuits false.
    assert decide(UNWIRED_UNIT_COVERAGE) is False


def test_false_when_only_coverage_changed_is_unwired():
    # unit-coverage is wired; the first operand is true, so only the second operand can fail it.
    assert decide(UNWIRED_COVERAGE_CHANGED) is False


def test_false_on_empty_text():
    assert decide("") is False
