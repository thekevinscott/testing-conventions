"""Colocated unit tests for the per-step `CLI_COMMAND` scan (isolation — only the unit under test).

Fixtures are two-step `steps:` lists at the reusable workflow's indentation, so a scan that runs
forward from the fallback line instead of bounding each step reads the neighbour's env line.
"""
from checks.hermetic_wired.cli_command_env import ENV_VALUE, step_blocks, step_name, unwired_steps

WIRED_STEP = (
    "      - name: Check lint\n"
    "        env:\n"
    f"          {ENV_VALUE}\n"
    "        run: ${CLI_COMMAND:-npx} unit lint\n"
)
BARE_STEP = (
    "      - name: Check colocated-test\n"
    "        run: ${CLI_COMMAND:-npx} unit colocated-test\n"
)
JOB_TAIL = "  packaging:\n    steps:\n      - run: echo hi\n"


def test_bounds_each_list_item_to_its_own_lines():
    blocks = step_blocks(WIRED_STEP + BARE_STEP)
    assert len(blocks) == 2
    assert ENV_VALUE in blocks[0]
    assert ENV_VALUE not in blocks[1]


def test_a_step_ends_at_the_job_level_key_that_closes_the_steps_list():
    blocks = step_blocks(BARE_STEP + JOB_TAIL)
    assert "packaging" not in blocks[0]


def test_reads_a_steps_name():
    assert step_name(BARE_STEP) == "Check colocated-test"


def test_falls_back_to_the_opening_line_of_an_unnamed_step():
    assert step_name("      - run: ${CLI_COMMAND:-npx} unit lint\n") == "run: ${CLI_COMMAND:-npx} unit lint"


def test_names_only_the_fallback_step_missing_its_own_env_line():
    assert unwired_steps(WIRED_STEP + BARE_STEP) == ["Check colocated-test"]


def test_a_neighbouring_steps_env_line_does_not_wire_the_next_step():
    assert unwired_steps(WIRED_STEP + BARE_STEP + JOB_TAIL) == ["Check colocated-test"]


def test_ignores_steps_that_never_run_the_fallback():
    assert unwired_steps("      - name: Checkout\n        uses: actions/checkout@v6\n") == []


def test_an_env_value_read_from_anywhere_but_detect_leaves_the_step_unwired():
    hardcoded = WIRED_STEP.replace(ENV_VALUE, "CLI_COMMAND: ./hermetic-cli/testing-conventions")
    assert unwired_steps(hardcoded) == ["Check lint"]
