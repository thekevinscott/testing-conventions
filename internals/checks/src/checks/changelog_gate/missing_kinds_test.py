"""Colocated unit tests for the owed-fragment decision (isolation — pure, no I/O)."""
from checks.changelog_gate.missing_kinds import missing_kinds


def test_missing_kinds_is_empty_when_both_fragments_are_added():
    assert missing_kinds(
        [
            "packages/rust/changelog.d/2026-08-30-a-fix.md",
            "packages/rust/migrations.d/2026-08-30-a-fix.md",
        ],
        "packages/rust",
    ) == []


def test_missing_kinds_names_migrations_when_only_a_changelog_fragment_is_added():
    assert missing_kinds(
        ["packages/rust/changelog.d/2026-08-30-a-fix.md"], "packages/rust"
    ) == ["migrations"]


def test_missing_kinds_names_changelog_when_only_a_migrations_fragment_is_added():
    assert missing_kinds(
        ["packages/rust/migrations.d/2026-08-30-a-fix.md"], "packages/rust"
    ) == ["changelog"]


def test_missing_kinds_names_both_when_nothing_is_added():
    assert missing_kinds([], "packages/rust") == ["changelog", "migrations"]


def test_missing_kinds_ignores_a_fragment_added_under_another_package():
    assert missing_kinds(
        [
            "packages/node/changelog.d/2026-08-30-a-fix.md",
            "packages/node/migrations.d/2026-08-30-a-fix.md",
        ],
        "packages/rust",
    ) == ["changelog", "migrations"]


def test_missing_kinds_ignores_a_malformed_fragment_name():
    assert missing_kinds(["packages/rust/changelog.d/a-fix.md"], "packages/rust") == [
        "changelog",
        "migrations",
    ]


def test_missing_kinds_ignores_the_directory_readme():
    assert missing_kinds(["packages/rust/changelog.d/README.md"], "packages/rust") == [
        "changelog",
        "migrations",
    ]
