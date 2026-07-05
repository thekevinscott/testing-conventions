"""End-to-end test for check_rolling_release_wired: runs `__main__` via runpy.

Covers `main`, both file reads, the file-presence branch, and the `__main__` guard against real
fixture files — a gated move-major-tag + clean release (pass), and a missing move-major-tag +
inline-move release (fail) — asserting exit code and printed lines each way.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_rolling_release_wired.py"

GATED = "on:\n  workflow_run:\n    workflows: [Release]\nconcurrency:\n  group: move-major-tag\n"


def run(argv):
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name="__main__")
        return 0
    except SystemExit as exit_:
        return exit_.code or 0
    finally:
        sys.argv = old


def test_e2e_passes_when_move_tag_gated_and_release_clean(tmp_path, capsys):
    move_tag = tmp_path / "move-major-tag.yml"
    move_tag.write_text(GATED)
    release = tmp_path / "release.yml"
    release.write_text("run: npm publish\n")
    assert run(["prog", str(move_tag), str(release)]) == 0
    assert "gated move-major-tag workflow" in capsys.readouterr().out


def test_e2e_fails_when_move_tag_absent_and_release_moves_inline(tmp_path, capsys):
    move_tag = tmp_path / "does-not-exist.yml"  # never created -> presence branch is False
    release = tmp_path / "release.yml"
    release.write_text("run: git tag -f v0 $SHA\n")
    assert run(["prog", str(move_tag), str(release)]) == 1
    out = capsys.readouterr().out
    assert "no dedicated advance workflow" in out
    assert "inline" in out
