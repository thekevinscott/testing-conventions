"""Colocated unit tests for the rolling-release-wired check (isolation — no `CliRunner`).

The two decisions are covered in `decide_test.py`; here the `cli` command is driven through its
`.callback` — once with both files present and clean, once with the move-tag file absent and an
inline-moving release — so both the `exists()` and the `errors` branches run. The raise path is
asserted against the propagated exception's `.message`.
"""
from checks.rolling_release_wired.cli import DEFAULT_MOVE_TAG, DEFAULT_RELEASE, cli

GATED = "on:\n  workflow_run:\n    workflows: [Release]\nconcurrency:\n  group: move-major-tag\n"


def test_declares_the_move_tag_and_release_arguments_with_their_defaults():
    move_tag_arg, release_arg = cli.params
    assert move_tag_arg.name == "move_tag"
    assert move_tag_arg.default == DEFAULT_MOVE_TAG
    assert release_arg.name == "release"
    assert release_arg.default == DEFAULT_RELEASE


def test_command_echoes_when_move_tag_gated_and_release_clean(tmp_path, capsys):
    move_tag = tmp_path / "move-major-tag.yml"
    move_tag.write_text(GATED)
    release = tmp_path / "release.yml"
    release.write_text("run: npm publish\n")
    cli.callback(move_tag=str(move_tag), release=str(release))
    assert "gated move-major-tag workflow" in capsys.readouterr().out


def test_command_raises_when_move_tag_absent_and_release_moves_inline(tmp_path):
    move_tag = tmp_path / "does-not-exist.yml"  # never created -> the exists() branch is False
    release = tmp_path / "release.yml"
    release.write_text("run: git tag -f v0 $SHA\n")
    try:
        cli.callback(move_tag=str(move_tag), release=str(release))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "no dedicated advance workflow" in error.message
        assert "inline" in error.message
    else:
        raise AssertionError("an absent move-tag file plus an inline-moving release must raise")
