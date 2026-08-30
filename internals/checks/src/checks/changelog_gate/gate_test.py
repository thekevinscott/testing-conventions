"""Colocated unit tests for the changelog-gate orchestration (isolation — injected git ops).

The three git reads are injected as hand-rolled fakes, so the orchestration is exercised without
a repo or a subprocess. Each fake records the SHAs it saw, which pins that `run` threads its own
arguments through rather than reading some other range.
"""
from checks.changelog_gate.gate import run


def _ops(changed=(), added=(), messages=""):
    """Fakes for the three git reads, each recording the (base, head) range it was asked for."""
    seen = []

    def changed_files(base, head):
        seen.append(("changed", base, head))
        return list(changed)

    def added_files(base, head):
        seen.append(("added", base, head))
        return list(added)

    def commit_messages(base, head):
        seen.append(("messages", base, head))
        return messages

    return {
        "changed_files": changed_files,
        "added_files": added_files,
        "commit_messages": commit_messages,
        "seen": seen,
    }


def _run(base="base", head="head", **kwargs):
    ops = _ops(**kwargs)
    seen = ops.pop("seen")
    return run(base, head, **ops), seen


def test_a_skip_line_bypasses_the_gate(capsys):
    code, _ = _run(
        changed=["packages/rust/src/lib.rs"],
        messages="refactor\n\nskip-changelog: pure rename\n",
    )
    assert code == 0
    assert "bypassing changelog enforcement" in capsys.readouterr().out


def test_a_skip_line_is_read_from_the_commit_range_under_test():
    _, seen = _run(base="abc", head="def", messages="skip-changelog: x\n")
    assert ("messages", "abc", "def") in seen


def test_no_package_source_changed_is_a_pass(capsys):
    code, _ = _run(changed=["internals/checks/src/checks/cli.py"])
    assert code == 0
    assert "No package source changed" in capsys.readouterr().out


def test_code_without_either_fragment_fails_naming_both_kinds(capsys):
    code, _ = _run(changed=["packages/rust/src/lib.rs"])
    out = capsys.readouterr().out
    assert code == 1
    assert "packages/rust/changelog.d/YYYY-MM-DD-<slug>.md" in out
    assert "packages/rust/migrations.d/YYYY-MM-DD-<slug>.md" in out
    assert "::error::" in out


def test_code_with_both_fragments_passes(capsys):
    code, _ = _run(
        changed=[
            "packages/rust/src/lib.rs",
            "packages/rust/changelog.d/2026-08-30-a-fix.md",
            "packages/rust/migrations.d/2026-08-30-a-fix.md",
        ],
        added=[
            "packages/rust/changelog.d/2026-08-30-a-fix.md",
            "packages/rust/migrations.d/2026-08-30-a-fix.md",
        ],
    )
    assert code == 0
    assert "::error::" not in capsys.readouterr().out


def test_code_with_only_a_changelog_fragment_fails_naming_migrations(capsys):
    code, _ = _run(
        changed=["packages/rust/src/lib.rs"],
        added=["packages/rust/changelog.d/2026-08-30-a-fix.md"],
    )
    out = capsys.readouterr().out
    assert code == 1
    assert "packages/rust/migrations.d/YYYY-MM-DD-<slug>.md" in out
    assert "packages/rust/changelog.d/YYYY-MM-DD-<slug>.md" not in out


def test_an_edited_fragment_does_not_satisfy_the_gate(capsys):
    # The fragment is in `changed` but not in `added`: editing an existing entry is not an entry.
    code, _ = _run(
        changed=[
            "packages/rust/src/lib.rs",
            "packages/rust/changelog.d/2026-08-30-a-fix.md",
            "packages/rust/migrations.d/2026-08-30-a-fix.md",
        ],
    )
    assert code == 1
    assert "::error::" in capsys.readouterr().out


def test_each_changed_package_is_judged_on_its_own_fragments(capsys):
    code, _ = _run(
        changed=["packages/rust/src/lib.rs", "packages/node/src/index.ts"],
        added=[
            "packages/rust/changelog.d/2026-08-30-a-fix.md",
            "packages/rust/migrations.d/2026-08-30-a-fix.md",
        ],
    )
    out = capsys.readouterr().out
    assert code == 1
    assert "packages/node" in out
    assert "packages/rust has code changes" not in out


def test_a_package_whose_only_changes_are_exempt_needs_no_fragment(capsys):
    code, _ = _run(changed=["packages/rust/CHANGELOG.md"])
    assert code == 0
    assert "::error::" not in capsys.readouterr().out


def test_a_malformed_fragment_name_fails_with_a_file_annotation(capsys):
    code, _ = _run(
        changed=["packages/rust/changelog.d/nope.md"],
        added=["packages/rust/changelog.d/nope.md"],
    )
    out = capsys.readouterr().out
    assert code == 1
    assert "::error file=packages/rust/changelog.d/nope.md::" in out
    assert "YYYY-MM-DD-<slug>.md" in out


def test_a_malformed_fragment_fails_even_with_no_package_source_changed(capsys):
    # The malformed-name diagnostic must not be short-circuited by the "nothing to enforce" exit.
    code, _ = _run(changed=["packages/rust/changelog.d/nope.md"])
    assert code == 1
    assert "No package source changed" not in capsys.readouterr().out


def test_the_diff_is_read_over_the_commit_range_under_test():
    _, seen = _run(base="abc", head="def", changed=["packages/rust/src/lib.rs"])
    assert ("changed", "abc", "def") in seen
    assert ("added", "abc", "def") in seen


def test_a_clean_run_says_the_fragments_are_present(capsys):
    code, _ = _run(
        changed=["packages/rust/src/lib.rs"],
        added=[
            "packages/rust/changelog.d/2026-08-30-a-fix.md",
            "packages/rust/migrations.d/2026-08-30-a-fix.md",
        ],
    )
    assert code == 0
    assert "changelog and migrations fragments present" in capsys.readouterr().out
