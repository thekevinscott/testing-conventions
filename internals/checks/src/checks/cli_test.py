"""Colocated unit test for the group: each check is registered as a subcommand (isolation).

Reads the composed group's command table — no file I/O, no dispatch. The commands' behavior is
covered by the e2e suite under `tests/`.
"""
from checks.cli import main


def test_build_command_wired_is_registered():
    assert "build-command-wired" in main.commands
