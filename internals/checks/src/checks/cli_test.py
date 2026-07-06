"""Colocated unit test for the group: each check is registered as a subcommand (isolation).

Reads the composed group's command table — no file I/O, no dispatch. The commands' behavior is
covered by the e2e suite under `tests/`.
"""
from checks.cli import main


def test_build_command_wired_is_registered():
    assert "build-command-wired" in main.commands


def test_colocated_rust_wired_is_registered():
    assert "colocated-rust-wired" in main.commands


def test_coverage_package_root_wired_is_registered():
    assert "coverage-package-root-wired" in main.commands


def test_coverage_rust_wired_is_registered():
    assert "coverage-rust-wired" in main.commands


def test_detect_routes_python_is_registered():
    assert "detect-routes-python" in main.commands


def test_diff_scoped_wired_is_registered():
    assert "diff-scoped-wired" in main.commands


def test_e2e_verify_checks_out_pr_head_is_registered():
    assert "e2e-verify-checks-out-pr-head" in main.commands


def test_e2e_verify_scope_wired_is_registered():
    assert "e2e-verify-scope-wired" in main.commands


def test_e2e_verify_wired_is_registered():
    assert "e2e-verify-wired" in main.commands


def test_gates_wired_is_registered():
    assert "gates-wired" in main.commands


def test_github_helpers_wired_is_registered():
    assert "github-helpers-wired" in main.commands


def test_isolation_wired_is_registered():
    assert "isolation-wired" in main.commands


def test_mutation_package_root_wired_is_registered():
    assert "mutation-package-root-wired" in main.commands


def test_mutation_wired_is_registered():
    assert "mutation-wired" in main.commands


def test_packaging_build_wired_is_registered():
    assert "packaging-build-wired" in main.commands


def test_packaging_package_root_wired_is_registered():
    assert "packaging-package-root-wired" in main.commands


def test_rolling_release_wired_is_registered():
    assert "rolling-release-wired" in main.commands


def test_rust_toolchain_wired_is_registered():
    assert "rust-toolchain-wired" in main.commands


def test_wiring_detect_action_is_registered():
    assert "wiring-detect-action" in main.commands


def test_wiring_e2e_default_on_is_registered():
    assert "wiring-e2e-default-on" in main.commands


def test_wiring_packaging_default_on_is_registered():
    assert "wiring-packaging-default-on" in main.commands
