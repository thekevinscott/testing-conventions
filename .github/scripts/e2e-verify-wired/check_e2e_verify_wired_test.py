"""Colocated unit tests for check_e2e_verify_wired.

Unit-level: the pure text inspection over crafted constants (no I/O). Both the input and the
command are required, so each is dropped independently to prove the AND logic. `main` and the
`__main__` guard are covered by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_e2e_verify_wired as m  # noqa: E402

INPUT = "  run_e2e:\n    description: 'Force the freshness gate on'\n"
COMMAND = "run: npx -y testing-conventions e2e verify \"$PACKAGE_ROOT\"\n"


def test_both_input_and_command_present_reports_no_missing_wiring():
    assert m.find_missing_wiring(INPUT + COMMAND) is None


def test_missing_run_e2e_input_is_reported():
    msg = m.find_missing_wiring(COMMAND)
    assert msg is not None
    assert "#173" in msg


def test_missing_e2e_verify_command_is_reported():
    assert m.find_missing_wiring(INPUT) is not None


def test_neither_present_is_reported():
    assert m.find_missing_wiring("run: unit coverage --language python\n") is not None


def test_path_from_argv_uses_default_when_no_argument():
    assert m.path_from_argv(["prog"], "the-default") == "the-default"


def test_path_from_argv_prefers_the_explicit_argument():
    assert m.path_from_argv(["prog", "other.yml"], "the-default") == "other.yml"
