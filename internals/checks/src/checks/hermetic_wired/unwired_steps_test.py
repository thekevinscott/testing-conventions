"""Colocated unit tests for the per-step `CLI_COMMAND` scan (isolation — only the unit under test).

Fixtures put a wired step directly above a bare one, so a scan that runs forward from the
fallback line instead of bounding each step reads the neighbour's env line.
"""
from checks.hermetic_wired.unwired_steps import ENV_VALUE, unwired_steps

LAUNCHER = 'npm --prefix "$RUNNER_TEMP" exec --yes --'

WIRED_STEP = (
    "      - name: Check lint\n"
    "        env:\n"
    f"          {ENV_VALUE}\n"
    f'        run: ${{CLI_COMMAND:-{LAUNCHER}}} unit lint\n'
)
BARE_STEP = (
    "      - name: Check colocated-test\n"
    f'        run: ${{CLI_COMMAND:-{LAUNCHER}}} unit colocated-test\n'
)
JOB_TAIL = "  packaging:\n    steps:\n      - run: echo hi\n"


def test_names_only_the_fallback_step_missing_its_own_env_line():
    assert unwired_steps(WIRED_STEP + BARE_STEP) == ["Check colocated-test"]


def test_a_neighbouring_steps_env_line_does_not_wire_the_next_step():
    assert unwired_steps(WIRED_STEP + BARE_STEP + JOB_TAIL) == ["Check colocated-test"]


def test_ignores_steps_that_never_run_the_fallback():
    assert unwired_steps("      - name: Checkout\n        uses: actions/checkout@v6\n") == []


def test_an_env_value_read_from_anywhere_but_detect_leaves_the_step_unwired():
    hardcoded = WIRED_STEP.replace(ENV_VALUE, "CLI_COMMAND: ./hermetic-cli/testing-conventions")
    assert unwired_steps(hardcoded) == ["Check lint"]
