"""Colocated unit tests for the step-name read (isolation — only the unit under test)."""
from checks.hermetic_wired.step_name import step_name

LAUNCHER = 'npm --prefix "$RUNNER_TEMP" exec --yes --'

NAMED_STEP = (
    "      - name: Check colocated-test\n"
    f'        run: ${{CLI_COMMAND:-{LAUNCHER}}} unit colocated-test\n'
)


def test_reads_a_steps_name():
    assert step_name(NAMED_STEP) == "Check colocated-test"


def test_falls_back_to_the_opening_line_of_an_unnamed_step():
    line = f'      - run: ${{CLI_COMMAND:-{LAUNCHER}}} unit lint\n'
    assert step_name(line) == line.strip().removeprefix('- ')
