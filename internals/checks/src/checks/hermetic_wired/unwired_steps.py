"""The per-step `CLI_COMMAND` scan: the env value is step-local, so a step running the fallback
without its own env line expands to the published binary while the fallback text survives."""
from __future__ import annotations

from checks.hermetic_wired.cli_command_env import step_blocks
from checks.hermetic_wired.step_name import step_name

ENV_VALUE = "CLI_COMMAND: ${{ needs.detect.outputs.cli_command }}"
FALLBACK = "${CLI_COMMAND:-"


def unwired_steps(text: str) -> list[str]:
    """Names of the steps running the fallback without their own `CLI_COMMAND` env line."""
    running = [block for block in step_blocks(text) if FALLBACK in block]
    return [step_name(block) for block in running if ENV_VALUE not in block]
