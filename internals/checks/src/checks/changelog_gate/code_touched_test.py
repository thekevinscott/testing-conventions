"""Colocated unit tests for the source-change decision (isolation — pure, no I/O).

The per-path exemption arms are pinned beside `is_exempt.py`; here the wiring is what matters:
scoping to the package, the `not is_exempt` filter, and source riding along with exempt paths.
"""
from checks.changelog_gate.code_touched import code_touched


def test_code_touched_is_true_for_package_source():
    assert code_touched(["packages/rust/src/lib.rs"], "packages/rust") is True


def test_code_touched_ignores_another_package():
    assert code_touched(["packages/node/src/index.ts"], "packages/rust") is False


def test_code_touched_ignores_an_exempt_path():
    assert code_touched(["packages/rust/CHANGELOG.md"], "packages/rust") is False


def test_code_touched_is_true_when_source_rides_along_with_exempt_paths():
    assert code_touched(
        ["packages/rust/CHANGELOG.md", "packages/rust/src/lib.rs"], "packages/rust"
    ) is True
