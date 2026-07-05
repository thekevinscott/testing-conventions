"""End-to-end tests for the detect-routes-python helper: the real `__main__` entry, no mocks.

Runs the script's `__main__` entry point in-process via `runpy`, passing an `isolation_languages`
value as `argv[1]` exactly as the workflow does. Covers `main`, the argv read, and the `__main__`
guard. A value that routes Python in exits 0; one that does not exits 1. `argv[0]` is a bogus,
non-JSON token so a mutant that reads it instead of `argv[1]` changes the outcome.

The last two tests pin the `if __name__ == "__main__":` guard's `==` against mutation: a distinct
runtime-built "__main__" (kills `is`) and a name sorting below "__main__" (kills `<=`).
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_detect_routes_python.py"

ARGV0 = "prog-not-json"
SUCCESS = "Python routed into the unit-lint matrix"


def run_named(run_name, argv):
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name=run_name)
        return 0
    except SystemExit as exit_:
        return exit_.code or 0
    finally:
        sys.argv = old


def run(argv):
    return run_named("__main__", argv)


def test_e2e_passes_when_python_is_routed_in(capsys):
    assert run([ARGV0, '["python","rust"]']) == 0
    out = capsys.readouterr().out
    assert 'isolation_languages=["python","rust"]' in out
    assert SUCCESS in out


def test_e2e_fails_when_python_is_absent(capsys):
    assert run([ARGV0, '["rust"]']) == 1
    assert "::error::" in capsys.readouterr().out


def test_e2e_guard_fires_for_a_distinct_main_object(capsys):
    # A runtime-built "__main__", equal by value but a distinct object: the guard's `==` fires
    # (main runs, prints the success line), where a mutated `is` would not.
    distinct_main = "".join(["__main", "__"])
    run_named(distinct_main, [ARGV0, '["python"]'])
    assert SUCCESS in capsys.readouterr().out


def test_e2e_guard_skips_for_a_name_below_main(capsys):
    # A run name sorting strictly below "__main__": the guard's `==` skips main, where a mutated
    # `<=` would fire it — so the success line is absent.
    run_named("__lain__", [ARGV0, '["python"]'])
    assert SUCCESS not in capsys.readouterr().out
