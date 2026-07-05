"""Colocated unit tests for check_github_helpers_wired.

Unit-level: the pure `find_missing_arm`, exercised in isolation over crafted workflow text (no
file reads, no subprocess, no network). `main` + the file read + the `__main__` guard are covered
by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_github_helpers_wired as m  # noqa: E402

# A workflow text that wires all five arms, in the diff-scoped `--base` form the gate uses.
WIRED = """
for rule in "unit colocated-test" "unit lint" "unit coverage" \\
            "integration lint" "unit mutation --base origin/main"; do
  npx -y testing-conventions $rule --language python "$dir"
done
"""


def _wired_without(arm: str) -> str:
    """WIRED with a single arm's phrase removed, so exactly that arm reads as missing."""
    return WIRED.replace(arm, "")


def test_fully_wired_text_has_no_missing_arm():
    assert m.find_missing_arm(WIRED) is None


def test_missing_colocated_test_is_named():
    assert m.find_missing_arm(_wired_without("unit colocated-test")) == "unit colocated-test"


def test_missing_unit_lint_is_named():
    assert m.find_missing_arm(_wired_without("unit lint")) == "unit lint"


def test_missing_unit_coverage_is_named():
    assert m.find_missing_arm(_wired_without("unit coverage")) == "unit coverage"


def test_missing_integration_lint_is_named():
    assert m.find_missing_arm(_wired_without("integration lint")) == "integration lint"


def test_missing_mutation_arm_entirely_is_named():
    assert m.find_missing_arm(_wired_without("unit mutation --base origin/main")) == (
        "unit mutation --base"
    )


def test_whole_tree_mutation_without_base_flag_counts_as_missing():
    # A bare `unit mutation` (no `--base`) is the whole-tree form the gate does not use, so the
    # diff-scoped mutation arm must still read as missing.
    text = WIRED.replace("unit mutation --base origin/main", "unit mutation")
    assert m.find_missing_arm(text) == "unit mutation --base"


def test_first_missing_arm_wins_when_several_are_absent():
    # colocated-test is checked before unit lint, so it is the one reported.
    text = _wired_without("unit colocated-test").replace("unit lint", "")
    assert m.find_missing_arm(text) == "unit colocated-test"
