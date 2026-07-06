"""Colocated unit tests for the e2e-verify-wired decision (isolation — pure, no I/O).

A fixture missing the input and one missing the command in turn pin the `and` joining them; the
both-present fixture pins the true arm.
"""
from checks.e2e_verify_wired.decide import wires_e2e_verify

BOTH = "inputs:\n  run_e2e:\n    default: false\nrun: e2e verify\n"
ONLY_INPUT = "inputs:\n  run_e2e:\n    default: false\nrun: unit coverage\n"
ONLY_COMMAND = "inputs:\n  other:\n    default: false\nrun: e2e verify\n"


def test_true_when_both_the_input_and_command_are_present():
    assert wires_e2e_verify(BOTH) is True


def test_false_when_only_the_input_is_present():
    assert wires_e2e_verify(ONLY_INPUT) is False


def test_false_when_only_the_command_is_present():
    assert wires_e2e_verify(ONLY_COMMAND) is False


def test_false_on_empty_text():
    assert wires_e2e_verify("") is False
