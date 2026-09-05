"""Colocated unit tests for the fragment-gate exemptions (isolation — pure, no I/O).

Each exemption arm is asserted one at a time, so dropping any single arm fails here rather than
silently demanding a fragment for a docs-shaped edit.
"""
from checks.changelog_gate.is_exempt import is_exempt


def test_package_source_is_not_exempt():
    assert is_exempt("packages/rust/src/lib.rs", "packages/rust") is False


def test_a_manifest_at_the_package_root_is_not_exempt():
    assert is_exempt("packages/rust/Cargo.toml", "packages/rust") is False


def test_the_frozen_changelog_stub_is_exempt():
    assert is_exempt("packages/rust/CHANGELOG.md", "packages/rust") is True


def test_the_frozen_migrations_stub_is_exempt():
    assert is_exempt("packages/rust/MIGRATIONS.md", "packages/rust") is True


def test_a_changelog_fragment_is_exempt():
    assert is_exempt("packages/rust/changelog.d/2026-08-30-a-fix.md", "packages/rust") is True


def test_a_migrations_fragment_is_exempt():
    assert is_exempt("packages/rust/migrations.d/2026-08-30-a-fix.md", "packages/rust") is True


def test_an_e2e_attestation_receipt_is_exempt():
    assert is_exempt("packages/rust/e2e-attestations/main.json", "packages/rust") is True


def test_a_suite_tier_test_is_exempt():
    assert is_exempt("packages/rust/tests/co_change.rs", "packages/rust") is True


def test_a_colocated_python_unit_test_is_exempt():
    assert is_exempt("packages/python/thing_test.py", "packages/python") is True


def test_a_colocated_typescript_unit_test_is_exempt():
    assert is_exempt("packages/node/src/thing.test.ts", "packages/node") is True


def test_a_colocated_typescript_spec_file_is_exempt():
    assert is_exempt("packages/node/src/thing.spec.ts", "packages/node") is True
