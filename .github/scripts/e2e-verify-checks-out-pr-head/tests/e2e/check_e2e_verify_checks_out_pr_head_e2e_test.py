"""End-to-end test for check_e2e_verify_checks_out_pr_head: runs `__main__` via runpy.

Covers `main`, the file read, and the `__main__` guard against real fixture files — one pinning
the PR head inside the `e2e-verify` job, one that does not — asserting exit code and printed line.
The two guard tests pin the `__name__ == "__main__"` comparison so mutating it (to `is` or `<=`)
is caught.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_e2e_verify_checks_out_pr_head.py"

BROKEN = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "  packaging:\n"
)


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


def test_e2e_passes_when_pin_present_in_job(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(
        "  e2e-verify:\n"
        "    steps:\n"
        "      - uses: actions/checkout@v6\n"
        "        with:\n"
        "          ref: ${{ github.event.pull_request.head.sha || github.sha }}\n"
        "  packaging:\n"
    )
    assert run(["prog", str(wf)]) == 0
    assert "checks out the PR head commit" in capsys.readouterr().out


def test_e2e_fails_when_pin_absent(tmp_path, capsys):
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
