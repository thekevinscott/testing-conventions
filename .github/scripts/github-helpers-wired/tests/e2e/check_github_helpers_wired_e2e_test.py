"""End-to-end tests for the github-helpers-wired check: real files, no mocks.

Runs the script's entry point in-process via `runpy` against temp workflow fixtures and the real
`dogfood-github-helpers.yml`, so the whole read -> assert -> report path stays on the
measured-coverage path. `sys.argv` and the working directory are saved and restored in `finally`,
never with pytest's `monkeypatch` fixture (the integration-lint gate forbids it).

The `run_name` the module is executed under is varied deliberately to exercise the
`if __name__ == "__main__"` guard from both sides: a name that sorts before `"__main__"` and a
distinct-but-equal-valued `"__main__"` string pin the guard's `==` so a `<=` or `is` mutant is
caught, and an empty / single-element `argv` pins the `len(argv) > 1` default-path branch.
"""
import os
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_github_helpers_wired.py"
REPO_ROOT = Path(__file__).resolve().parents[5]
REAL_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "dogfood-github-helpers.yml"

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
import check_github_helpers_wired as m  # noqa: E402

# A workflow snippet wiring all five arms in the diff-scoped `--base` form the gate uses.
WIRED = (
    'for rule in "unit colocated-test" "unit lint" "unit coverage" '
    '"integration lint" "unit mutation --base origin/main"; do\n'
    '  npx -y testing-conventions $rule --language python "$dir"\n'
    "done\n"
)


def run(argv, run_name="__main__", cwd=None):
    """Run the script in-process under `run_name`, restoring argv/cwd afterward.

    Returns the exit code (0 when the module finishes without raising `SystemExit`, e.g. when the
    `__main__` guard is false).
    """
    old_argv = sys.argv
    old_cwd = os.getcwd()
    sys.argv = argv
    if cwd is not None:
        os.chdir(cwd)
    try:
        runpy.run_path(str(SCRIPT), run_name=run_name)
        return 0
    except SystemExit as exit_:
        return exit_.code or 0
    finally:
        sys.argv = old_argv
        os.chdir(old_cwd)


def test_e2e_passes_on_a_fully_wired_fixture(tmp_path, capsys):
    wf = tmp_path / "dogfood.yml"
    wf.write_text(WIRED)
    assert run(["check", str(wf)]) == 0
    assert "runs all five Python arms" in capsys.readouterr().out


def test_e2e_fails_on_a_fixture_missing_an_arm(tmp_path, capsys):
    wf = tmp_path / "broken.yml"
    wf.write_text(WIRED.replace('"integration lint" ', ""))
    assert run(["check", str(wf)], cwd=tmp_path) == 1
    out = capsys.readouterr().out
    assert "::error::" in out
    assert "integration lint" in out


def test_e2e_passes_against_the_real_dogfood_workflow(capsys):
    assert run(["check", str(REAL_WORKFLOW)]) == 0
    assert "runs all five Python arms" in capsys.readouterr().out


def test_e2e_single_arg_uses_the_default_path(capsys):
    # argv of length 1 (no path): `len(argv) > 1` is false, so the default path is read. Run from
    # the repo root so the default relative path resolves. Pins `>` against `>=`/`<=`/`==`.
    assert run(["check"], cwd=REPO_ROOT) == 0
    assert "runs all five Python arms" in capsys.readouterr().out


def test_e2e_empty_argv_uses_the_default_path(capsys):
    # An empty argv (length 0): `len(argv) > 1` is false, so the default path is read. `runpy`
    # itself requires argv[0], so `main` is called directly here. A `!=` mutant would be true for
    # length 0 and index argv[1], so this pins `>` against `!=`. Run from the repo root.
    old_cwd = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        assert m.main([]) == 0
    finally:
        os.chdir(old_cwd)
    assert "runs all five Python arms" in capsys.readouterr().out


def test_e2e_guard_stays_shut_for_a_name_sorting_before_dunder_main(tmp_path, capsys):
    # "AAA" < "__main__", so `==` keeps the guard shut and main never runs. A `<=` (or `<`) mutant
    # would open the guard and print, so an empty stdout kills it.
    wf = tmp_path / "dogfood.yml"
    wf.write_text(WIRED)
    assert run(["check", str(wf)], run_name="AAA") == 0
    assert capsys.readouterr().out == ""


def test_e2e_guard_opens_for_a_distinct_equal_valued_main_string(tmp_path, capsys):
    # A "__main__" built at runtime is equal in value but a distinct object, so `==` opens the
    # guard and main runs, while an `is` mutant would keep it shut and print nothing.
    wf = tmp_path / "dogfood.yml"
    wf.write_text(WIRED)
    distinct_main = "".join(list("__main__"))  # equal value, distinct object (not interned)
    assert run(["check", str(wf)], run_name=distinct_main) == 0
    assert "runs all five Python arms" in capsys.readouterr().out
