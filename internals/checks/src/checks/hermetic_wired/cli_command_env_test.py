"""Colocated unit tests for the step-block bounding (isolation — only the unit under test).

Fixtures are two-step `steps:` lists at the reusable workflow's indentation, so a scan that runs
forward from a needle line instead of bounding each step reads the neighbour's lines.
"""
from checks.hermetic_wired.cli_command_env import step_blocks

ENV_LINE = "CLI_COMMAND: ${{ needs.detect.outputs.cli_command }}"
LAUNCHER = 'npm --prefix "$RUNNER_TEMP" exec --yes --'

WIRED_STEP = (
    "      - name: Check lint\n"
    "        env:\n"
    f"          {ENV_LINE}\n"
    f'        run: ${{CLI_COMMAND:-{LAUNCHER}}} unit lint\n'
)
BARE_STEP = (
    "      - name: Check colocated-test\n"
    f'        run: ${{CLI_COMMAND:-{LAUNCHER}}} unit colocated-test\n'
)
JOB_TAIL = "  packaging:\n    steps:\n      - run: echo hi\n"


def test_bounds_each_list_item_to_its_own_lines():
    blocks = step_blocks(WIRED_STEP + BARE_STEP)
    assert len(blocks) == 2
    assert ENV_LINE in blocks[0]
    assert ENV_LINE not in blocks[1]


def test_a_step_ends_at_the_job_level_key_that_closes_the_steps_list():
    blocks = step_blocks(BARE_STEP + JOB_TAIL)
    assert "packaging" not in blocks[0]
