"""Colocated unit tests for check_mutation_wired.

Unit-level: the pure text inspection, exercised in isolation over crafted WIRED / UNWIRED
constants (no file reads, no subprocess, no network). `main` and the `__main__` guard are covered
by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_mutation_wired as m  # noqa: E402

WIRED = 'run: npx -y testing-conventions unit mutation --language "$LANGUAGE" --base "$BASE"\n'
UNWIRED = 'run: npx -y testing-conventions unit coverage --language "$LANGUAGE"\n'


def test_wired_workflow_reports_no_missing_wiring():
    assert m.find_missing_wiring(WIRED) is None


def test_unwired_workflow_reports_the_missing_gate():
    msg = m.find_missing_wiring(UNWIRED)
    assert msg is not None
    assert "unit mutation" in msg
    assert "#204" in msg


def test_bare_mutation_word_without_language_flag_is_not_wired():
    # `--language` is load-bearing: the shipped rule is fanned per-language.
    assert m.find_missing_wiring("some prose mentioning mutation testing\n") is not None


def test_path_from_argv_uses_default_when_no_argument():
    assert m.path_from_argv(["prog"], "the-default") == "the-default"


def test_path_from_argv_prefers_the_explicit_argument():
    assert m.path_from_argv(["prog", "other.yml"], "the-default") == "other.yml"
