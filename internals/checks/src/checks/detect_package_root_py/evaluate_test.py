"""Colocated unit tests for the detect-package-root-py decision (isolation — pure, no I/O).

The correct outputs are hardcoded here (not imported from config), so a mutated config constant
makes a "matches" case mismatch and fails — killing the constant's mutant. The single
`actual != expected` comparison is pinned from both sides — a lexically greater and a lexically
lesser wrong input.
"""
from checks.detect_package_root_py.evaluate import evaluate

ROOT = ".github/selftest/monorepo/packages/py"
CONFIG = ".github/selftest/monorepo/packages/py/testing-conventions.toml"


def test_passes_when_every_output_matches():
    assert evaluate(ROOT, "uv", CONFIG) is None


def test_flags_a_package_root_greater_than_expected():
    greater = ROOT + "/src"  # lexically greater than the expected root
    assert greater > ROOT
    assert "package_root" in evaluate(greater, "uv", CONFIG)


def test_flags_a_package_root_less_than_expected():
    lesser = ".github/selftest/monorepo/packages"  # lexically less than the expected root
    assert lesser < ROOT
    assert "package_root" in evaluate(lesser, "uv", CONFIG)


def test_flags_a_wrong_python_env():
    assert "python_env" in evaluate(ROOT, "pip", CONFIG)


def test_flags_a_wrong_config():
    assert "config" in evaluate(ROOT, "uv", "testing-conventions.toml")
