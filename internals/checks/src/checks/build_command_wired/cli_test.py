"""Colocated unit tests for the build-command-wired predicate (isolation — no file reads)."""
from checks.build_command_wired.cli import wires_build_command


def test_true_when_the_build_command_output_is_referenced():
    assert wires_build_command("run: ${{ needs.detect.outputs.build_command }}") is True


def test_false_when_the_output_is_absent():
    assert wires_build_command("run: echo no build step here") is False


def test_false_on_empty_text():
    assert wires_build_command("") is False
