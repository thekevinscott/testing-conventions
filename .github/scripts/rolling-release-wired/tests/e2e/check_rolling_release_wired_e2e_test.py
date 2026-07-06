"""End-to-end test for check_rolling_release_wired: runs `__main__` via runpy.

Covers `main`, both file reads, the file-presence branch, and the `__main__` guard against real
fixture files — a gated move-major-tag + clean release (pass), and a missing move-major-tag +
inline-move release (fail) — asserting exit code and printed lines each way. The two guard tests
pin the `__name__ == "__main__"` comparison so mutating it (to `is` or `<=`) is caught.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_rolling_release_wired.py"

GATED = "on:\n  workflow_run:\n    workflows: [Release]\nconcurrency:\n  group: move-major-tag\n"


def run(argv, run_name="__main__"):
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name=run_name)
        return 0
    except SystemExit as exit_:
        return exit_.code or 0
    finally:
        sys.argv = old


def _broken(tmp_path):
    """A missing move-major-tag path + a release.yml that moves @v0 inline (main returns 1)."""
    move_tag = tmp_path / "does-not-exist.yml"  # never created -> presence branch is False
    release = tmp_path / "release.yml"
    release.write_text("run: git tag -f v0 $SHA\n")
    return str(move_tag), str(release)


def test_e2e_passes_when_move_tag_gated_and_release_clean(tmp_path, capsys):
    move_tag = tmp_path / "move-major-tag.yml"
    move_tag.write_text(GATED)
    release = tmp_path / "release.yml"
    release.write_text("run: npm publish\n")
    assert run(["prog", str(move_tag), str(release)]) == 0
    assert "gated move-major-tag workflow" in capsys.readouterr().out


def test_e2e_fails_when_move_tag_absent_and_release_moves_inline(tmp_path, capsys):
    move_tag, release = _broken(tmp_path)
    assert run(["prog", move_tag, release]) == 1
    out = capsys.readouterr().out
    assert "no dedicated advance workflow" in out
    assert "inline" in out


def test_e2e_guard_runs_main_only_for_real_dunder_main(tmp_path, capsys):
    # run_name equals "__main__" by content but is a distinct object (built at runtime), so the
    # guard's `==` runs main (exit 1 on the broken fixture); an `is` mutant would skip it.
    move_tag, release = _broken(tmp_path)
    runtime_name = "".join(["_", "_", "m", "a", "i", "n", "_", "_"])
    assert run(["prog", move_tag, release], run_name=runtime_name) == 1
    assert "::error::" in capsys.readouterr().out


def test_e2e_guard_skips_main_when_name_sorts_below_dunder_main(tmp_path, capsys):
    # run_name sorts lexicographically below "__main__": `==` is False so main is skipped, but a
    # `<=`/`<` mutant would run it — the exit code and empty output distinguish them.
    move_tag, release = _broken(tmp_path)
    assert run(["prog", move_tag, release], run_name="__aaaaaa__") == 0
    assert capsys.readouterr().out == ""
