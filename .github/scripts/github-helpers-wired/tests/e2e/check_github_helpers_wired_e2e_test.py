"""End-to-end tests for the github-helpers-wired check: real files, no mocks.

Runs the script's `__main__` entry in-process via `runpy` against temp workflow fixtures, so the
whole read -> assert -> report path (and the `__main__` guard) stays on the measured-coverage
path. Also runs it against the *real* `dogfood-github-helpers.yml` to prove the check holds today.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_github_helpers_wired.py"
REAL_WORKFLOW = (
    Path(__file__).resolve().parents[5]
    / ".github"
    / "workflows"
    / "dogfood-github-helpers.yml"
)

WIRED = (
    'for rule in "unit colocated-test" "unit lint" "unit coverage" '
    '"integration lint" "unit mutation --base origin/main"; do\n'
    '  npx -y testing-conventions $rule --language python "$dir"\n'
    "done\n"
)


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


def test_e2e_passes_on_a_fully_wired_fixture(tmp_path, capsys):
    wf = tmp_path / "dogfood.yml"
    wf.write_text(WIRED)
    assert run(["check", str(wf)]) == 0
    assert "runs all five Python arms" in capsys.readouterr().out


def test_e2e_fails_on_a_fixture_missing_an_arm(tmp_path, capsys):
    wf = tmp_path / "broken.yml"
    wf.write_text(WIRED.replace('"integration lint" ', ""))
    assert run(["check", str(wf)]) == 1
    out = capsys.readouterr().out
    assert "::error::" in out
    assert "integration lint" in out


def test_e2e_passes_against_the_real_dogfood_workflow(capsys):
    assert run(["check", str(REAL_WORKFLOW)]) == 0
    assert "runs all five Python arms" in capsys.readouterr().out


def test_e2e_default_path_resolves_from_the_repo_root(monkeypatch, capsys):
    # No argv[1]: exercises the DEFAULT_PATH branch, resolved cwd-relative from the repo root.
    monkeypatch.chdir(REAL_WORKFLOW.parents[2])
    assert run(["check"]) == 0
    assert "runs all five Python arms" in capsys.readouterr().out
