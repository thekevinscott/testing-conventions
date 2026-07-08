"""Colocated unit tests for the detect-package-root-ts decision (isolation — pure, no I/O).

The correct outputs are hardcoded here (not imported from config), so a mutated config constant
makes a "matches" case mismatch and fails — killing the constant's mutant. The single
`actual != expected` comparison is pinned from both sides — a lexically greater and a lexically
lesser wrong input — so a mutation to `<` / `>` / `==` / `is` changes an asserted outcome.
"""
from checks.detect_package_root_ts.evaluate import evaluate

ROOT = ".github/selftest/monorepo/packages/ts"
CONFIG = ".github/selftest/monorepo/packages/ts/testing-conventions.toml"


def test_passes_when_every_output_matches():
    assert evaluate(ROOT, "npm", "false", CONFIG) is None


def test_flags_a_package_root_greater_than_expected():
    greater = ROOT + "/src"  # lexically greater than the expected root
    assert greater > ROOT
    assert "package_root" in evaluate(greater, "npm", "false", CONFIG)


def test_flags_a_package_root_less_than_expected():
    lesser = ".github/selftest/monorepo/packages"  # lexically less than the expected root
    assert lesser < ROOT
    assert "package_root" in evaluate(lesser, "npm", "false", CONFIG)


def test_flags_a_wrong_package_manager():
    assert "ts_package_manager" in evaluate(ROOT, "pnpm", "false", CONFIG)


def test_flags_a_wrong_provision_rust():
    assert "provision_rust" in evaluate(ROOT, "npm", "true", CONFIG)


def test_flags_a_wrong_config():
    assert "config" in evaluate(ROOT, "npm", "false", "testing-conventions.toml")
