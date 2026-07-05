"""End-to-end test for check_diff_scoped_wired: runs the real `__main__` entry via runpy.

Covers `main`, the file read, and the `__main__` guard against real fixture files — one with both
`--base` checks and one missing them — asserting exit code and printed line each way. The two
guard tests pin the `__name__ == "__main__"` comparison so mutating it (to `is` or `<=`) is caught.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_diff_scoped_wired.py"

BROKEN = 'run: unit coverage --language python\n'


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


def test_e2e_passes_when_both_base_checks_present(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(
        'run: unit colocated-test --language python --base "$BASE"\n'
        'run: unit coverage --language python --base "$BASE"\n'
    )
    assert run(["prog", str(wf)]) == 0
    assert "diff-scoped --base checks" in capsys.readouterr().out


def test_e2e_fails_when_a_base_check_is_missing(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(BROKEN)
    assert run(["prog", str(wf)]) == 1
    assert "::error::" in capsys.readouterr().out


def test_e2e_guard_runs_main_only_for_real_dunder_main(tmp_path, capsys):
    # run_name equals "__main__" by content but is a distinct object (built at runtime), so the
    # guard's `==` runs main (exit 1 on the broken fixture); an `is` mutant would skip it.
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(BROKEN)
    runtime_name = "".join(["_", "_", "m", "a", "i", "n", "_", "_"])
    assert run(["prog", str(wf)], run_name=runtime_name) == 1
    assert "::error::" in capsys.readouterr().out


def test_e2e_guard_skips_main_when_name_sorts_below_dunder_main(tmp_path, capsys):
    # run_name sorts lexicographically below "__main__": `==` is False so main is skipped, but a
    # `<=`/`<` mutant would run it — the exit code and empty output distinguish them.
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(BROKEN)
    assert run(["prog", str(wf)], run_name="__aaaaaa__") == 0
    assert capsys.readouterr().out == ""
