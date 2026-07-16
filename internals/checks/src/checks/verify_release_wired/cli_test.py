"""Colocated unit tests for the verify-release-wired check (isolation — no `CliRunner`).

The decision is covered in `decide_test.py`; here the `cli` command is driven through its
`.callback` — once with a wired-and-gated file, once with an ungated one — so both the clean and
the raise branches run. The raise path is asserted against the propagated exception's `.message`.
"""
from checks.verify_release_wired.cli import DEFAULT_MOVE_TAG, cli

# A minimal wired-and-gated move-major-tag.yml: both verification steps present, both suite
# workflows dispatched, and a move job that needs the verify jobs. (The decision's branches are
# exhausted in decide_test; here we only need one clean sample and one ungated sample.)
WIRED = """\
jobs:
  verify-layout:
    steps:
      - run: uv run --project internals/checks tc-checks verify-release check-layout "$SHA"
  verify-suite:
    steps:
      - run: uv run --project internals/checks tc-checks verify-release dispatch-and-wait "$SHA" "$V" testing-conventions-selftest.yml dogfood.yml
  move-v0:
    needs: [verify-layout, verify-suite]
    steps:
      - run: python3 internals/move-major-tag/src/move_major_tag.py
"""


def test_declares_the_move_tag_argument_with_its_default():
    (argument,) = cli.params
    assert argument.name == "move_tag"
    assert argument.default == DEFAULT_MOVE_TAG


def test_command_echoes_when_the_move_is_gated_on_verification(tmp_path, capsys):
    move_tag = tmp_path / "move-major-tag.yml"
    move_tag.write_text(WIRED)
    cli.callback(move_tag=str(move_tag))
    assert "@v0 advances only after the version-pinned verification passes" in capsys.readouterr().out


def test_command_raises_when_the_file_is_absent(tmp_path):
    try:
        cli.callback(move_tag=str(tmp_path / "nope.yml"))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "no workflow advances @v0" in error.message
    else:
        raise AssertionError("an absent move-major-tag.yml must raise")


def test_command_raises_when_the_move_is_ungated(tmp_path):
    move_tag = tmp_path / "move-major-tag.yml"
    move_tag.write_text(WIRED.replace("    needs: [verify-layout, verify-suite]\n", ""))
    try:
        cli.callback(move_tag=str(move_tag))
    except Exception as error:  # noqa: BLE001
        assert "isn't gated on verification" in error.message
    else:
        raise AssertionError("an ungated move job must raise")
