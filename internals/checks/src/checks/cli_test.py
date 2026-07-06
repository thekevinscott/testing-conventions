"""Colocated unit test for the group: each check is registered as a subcommand (isolation).

Reads the composed group's command table — no file I/O, no dispatch. The commands' behavior is
covered by the e2e suite under `tests/`.
"""
from checks.cli import main


def test_build_command_wired_is_registered():
    assert "build-command-wired" in main.commands


def test_e2e_verify_scope_wired_is_registered():
    assert "e2e-verify-scope-wired" in main.commands


def test_gates_wired_is_registered():
    assert "gates-wired" in main.commands


def test_rust_toolchain_wired_is_registered():
    assert "rust-toolchain-wired" in main.commands
