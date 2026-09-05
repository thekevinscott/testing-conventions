"""Colocated unit tests for the changelog-gate vocabulary decisions (isolation — pure, no I/O)."""
from checks.changelog_gate.decide import KINDS, changed_packages, has_skip_line


def test_kinds_are_changelog_and_migrations():
    assert KINDS == ("changelog", "migrations")


def test_skip_line_is_found_anywhere_in_a_body():
    assert has_skip_line("tidy up\n\nskip-changelog: pure rename\n") is True


def test_skip_line_is_found_on_the_first_line():
    assert has_skip_line("skip-changelog: pure rename\n") is True


def test_skip_line_is_case_insensitive():
    assert has_skip_line("Skip-Changelog: pure rename\n") is True


def test_skip_line_must_start_its_line():
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


def test_changed_packages_ignores_the_packages_directory_itself():
    assert changed_packages(["packages"]) == []


def test_changed_packages_takes_a_file_sitting_at_a_package_root():
    assert changed_packages(["packages/rust/Cargo.toml"]) == ["packages/rust"]


def test_changed_packages_ignores_a_root_that_sorts_after_packages():
    # The first segment is compared for equality, not order.
    assert changed_packages(["tools/rust/lib.rs"]) == []
