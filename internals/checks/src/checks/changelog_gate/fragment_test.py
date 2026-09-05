"""Colocated unit tests for the fragment-path recogniser (isolation — pure, no I/O)."""
from checks.changelog_gate.fragment import fragment


def test_fragment_splits_a_path_into_package_kind_and_filename():
    assert fragment("packages/rust/changelog.d/2026-08-30-a-fix.md") == (
        "packages/rust",
        "changelog",
        "2026-08-30-a-fix.md",
    )


def test_fragment_recognises_the_migrations_kind():
    assert fragment("packages/node/migrations.d/2026-08-30-a-fix.md") == (
        "packages/node",
        "migrations",
        "2026-08-30-a-fix.md",
    )


def test_fragment_is_none_for_a_non_fragment_path():
    assert fragment("packages/rust/src/lib.rs") is None


def test_fragment_is_none_for_a_nested_path_under_a_fragment_dir():
    assert fragment("packages/rust/changelog.d/old/2026-08-30-a-fix.md") is None
