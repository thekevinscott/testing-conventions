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


def test_wiring_detect_action_is_registered():
    assert "wiring-detect-action" in main.commands


def test_wiring_packaging_default_on_is_registered():
    assert "wiring-packaging-default-on" in main.commands


def test_wiring_e2e_default_on_is_registered():
    assert "wiring-e2e-default-on" in main.commands


def test_detect_routes_python_is_registered():
    assert "detect-routes-python" in main.commands
