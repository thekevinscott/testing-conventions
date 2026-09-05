"""Colocated unit tests for the fragment-name screening (isolation — pure, no I/O)."""
from checks.changelog_gate.malformed_fragments import malformed_fragments


def test_malformed_fragments_flags_a_name_without_a_date():
    assert malformed_fragments(["packages/rust/changelog.d/a-fix.md"]) == [
        "packages/rust/changelog.d/a-fix.md"
    ]


def test_malformed_fragments_flags_an_uppercase_slug():
    assert malformed_fragments(["packages/rust/changelog.d/2026-08-30-A-Fix.md"]) == [
        "packages/rust/changelog.d/2026-08-30-A-Fix.md"
    ]


def test_malformed_fragments_flags_a_non_markdown_extension():
    assert malformed_fragments(["packages/rust/changelog.d/2026-08-30-a-fix.txt"]) == [
        "packages/rust/changelog.d/2026-08-30-a-fix.txt"
    ]


def test_malformed_fragments_accepts_a_well_formed_name():
    assert malformed_fragments(["packages/rust/changelog.d/2026-08-30-a-fix.md"]) == []


def test_malformed_fragments_accepts_the_directory_readme():
    # Each fragment dir carries a README.md explaining the convention; it is not an entry.
    assert malformed_fragments(["packages/rust/changelog.d/README.md"]) == []


def test_malformed_fragments_ignores_paths_that_are_not_fragments():
    assert malformed_fragments(["packages/rust/src/lib.rs"]) == []
