"""Colocated unit tests for the step chunking (isolation — pure text in/out)."""
from checks.pnpm_version_wired.pnpm_steps import pnpm_steps

PNPM_STEP = "      - uses: pnpm/action-setup@v5\n        with:\n          version: 11\n"
NODE_STEP = "      - uses: actions/setup-node@v6\n        with:\n          node-version: 24\n"


def test_returns_only_the_chunks_that_use_pnpm_action_setup():
    (chunk,) = pnpm_steps(PNPM_STEP + NODE_STEP)
    assert any("pnpm/action-setup" in line for line in chunk)


def test_a_chunk_ends_where_the_next_step_opens():
    (chunk,) = pnpm_steps(PNPM_STEP + NODE_STEP)
    assert not any("setup-node" in line for line in chunk)


def test_a_step_whose_uses_follows_its_if_line_is_one_chunk():
    text = (
        "      - if: matrix.language == 'typescript'\n"
        "        uses: pnpm/action-setup@v5\n"
        "        with:\n"
        "          version: 11\n"
    )
    (chunk,) = pnpm_steps(text)
    assert any("version: 11" in line for line in chunk)


def test_lines_before_the_first_step_belong_to_no_chunk():
    preamble = "on:\n  workflow_call:\n    inputs:\n      version:\n        type: string\n"
    assert pnpm_steps(preamble) == []
