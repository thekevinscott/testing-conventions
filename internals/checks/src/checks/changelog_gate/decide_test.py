"""Colocated unit tests for the changelog-gate decisions (isolation — pure, no I/O).

Each predicate is pinned against the path shapes the gate actually sees: package source, the
frozen pointer stubs, the fragment dirs themselves, and the test tiers. The stub/fragment/test
exemptions are asserted one at a time, so dropping any single arm of `code_touched`'s exemption
list fails here rather than silently demanding a fragment for a docs-shaped edit.
"""
from checks.changelog_gate.decide import (
    KINDS,
    changed_packages,
    code_touched,
    fragment,
    has_skip_line,
    malformed_fragments,
    missing_kinds,
)


def test_kinds_are_changelog_and_migrations():
    assert KINDS == ("changelog", "migrations")


def test_skip_line_is_found_anywhere_in_a_body():
    assert has_skip_line("tidy up\n\nskip-changelog: pure rename\n") is True


def test_skip_line_is_found_on_the_first_line():
    assert has_skip_line("skip-changelog: pure rename\n") is True


def test_skip_line_is_case_insensitive():
    assert has_skip_line("Skip-Changelog: pure rename\n") is True


def test_skip_line_must_start_its_line():
    # A mention inside prose is not the bypass — only a line that opens with the token.
    assert has_skip_line("we could skip-changelog: but we won't\n") is False


def test_no_skip_line_in_an_ordinary_body():
    assert has_skip_line("fix: correct the off-by-one\n") is False


def test_changed_packages_are_unique_and_sorted():
    assert changed_packages(
        [
            "packages/rust/src/lib.rs",
            "packages/node/src/index.ts",
            "packages/rust/Cargo.toml",
        ]
    ) == ["packages/node", "packages/rust"]


def test_changed_packages_ignores_paths_outside_the_packages_root():
    assert changed_packages(["internals/checks/src/checks/cli.py", "AGENTS.md"]) == []


def test_changed_packages_ignores_a_bare_file_directly_under_packages():
    assert changed_packages(["packages/README.md"]) == []


def test_code_touched_is_true_for_package_source():
    assert code_touched(["packages/rust/src/lib.rs"], "packages/rust") is True


def test_code_touched_ignores_another_package():
    assert code_touched(["packages/node/src/index.ts"], "packages/rust") is False


def test_code_touched_ignores_the_frozen_changelog_stub():
    assert code_touched(["packages/rust/CHANGELOG.md"], "packages/rust") is False


def test_code_touched_ignores_the_frozen_migrations_stub():
    assert code_touched(["packages/rust/MIGRATIONS.md"], "packages/rust") is False


def test_code_touched_ignores_changelog_fragments():
    assert code_touched(
        ["packages/rust/changelog.d/2026-08-30-a-fix.md"], "packages/rust"
    ) is False


def test_code_touched_ignores_migrations_fragments():
    assert code_touched(
        ["packages/rust/migrations.d/2026-08-30-a-fix.md"], "packages/rust"
    ) is False


def test_code_touched_ignores_a_colocated_python_unit_test():
    assert code_touched(["packages/python/thing_test.py"], "packages/python") is False


def test_code_touched_ignores_a_colocated_typescript_unit_test():
    assert code_touched(["packages/node/src/thing.test.ts"], "packages/node") is False


def test_code_touched_ignores_a_colocated_typescript_spec_file():
    assert code_touched(["packages/node/src/thing.spec.ts"], "packages/node") is False


def test_code_touched_ignores_a_rust_suite_test():
    assert code_touched(["packages/rust/tests/co_change.rs"], "packages/rust") is False


def test_code_touched_ignores_the_e2e_attestation_receipts():
    assert code_touched(
        ["packages/rust/e2e-attestations/main.json"], "packages/rust"
    ) is False


def test_code_touched_is_true_when_source_rides_along_with_exempt_paths():
    assert code_touched(
        ["packages/rust/CHANGELOG.md", "packages/rust/src/lib.rs"], "packages/rust"
    ) is True


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
    # The trailing segment forbids nesting, so a stray subdirectory is not a fragment.
    assert fragment("packages/rust/changelog.d/old/2026-08-30-a-fix.md") is None


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
    # A file that breaks the naming convention does not satisfy the gate.
    assert missing_kinds(["packages/rust/changelog.d/a-fix.md"], "packages/rust") == [
        "changelog",
        "migrations",
    ]


def test_missing_kinds_ignores_the_directory_readme():
    assert missing_kinds(["packages/rust/changelog.d/README.md"], "packages/rust") == [
        "changelog",
        "migrations",
    ]
