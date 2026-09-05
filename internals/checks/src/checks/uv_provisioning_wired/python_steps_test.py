"""Colocated unit tests for the python-arm step picker (isolation — pure text in/out)."""
from checks.uv_provisioning_wired.python_steps import python_steps


def test_python_steps_picks_only_python_gated_step_chunks():
    block = (
        "  unit-coverage:\n"
        "    steps:\n"
        "      - uses: actions/checkout@v6\n"
        "      - if: matrix.language == 'python'\n"
        "        run: uv sync\n"
        "      - if: matrix.language == 'typescript'\n"
        "        run: npm ci\n"
    )
    assert python_steps(block) == "      - if: matrix.language == 'python'\n        run: uv sync"


def test_python_steps_drops_comment_and_blank_lines_inside_a_chunk():
    block = (
        "      - if: matrix.language == 'python'\n"
        "        # provision with uv\n"
        "\n"
        "        run: uv sync\n"
    )
    assert python_steps(block) == "      - if: matrix.language == 'python'\n        run: uv sync"


def test_python_steps_is_empty_when_no_chunk_is_python_gated():
    assert python_steps("      - uses: actions/checkout@v6\n") == ""
