"""The single console entry point: the click group that composes the checks (#321).

Repo-only. `tc-checks` is one command; each check is a `@click.command()` in its own subfolder,
registered here as a subcommand. Adding a check is a folder plus one `add_command` line.
"""
from __future__ import annotations

import click

from checks.below_floor.cli import cli as below_floor
from checks.build_command_wired.cli import cli as build_command_wired
from checks.build_hermetic_cli.cli import cli as build_hermetic_cli
from checks.colocated_rust_red.cli import cli as colocated_rust_red
from checks.colocated_rust_wired.cli import cli as colocated_rust_wired
from checks.coverage_package_root_wired.cli import cli as coverage_package_root_wired
from checks.coverage_rust_red.cli import cli as coverage_rust_red
from checks.coverage_rust_wired.cli import cli as coverage_rust_wired
from checks.detect_package_root_py.cli import cli as detect_package_root_py
from checks.detect_package_root_ts.cli import cli as detect_package_root_ts
from checks.detect_routes_python.cli import cli as detect_routes_python
from checks.diff_scoped_wired.cli import cli as diff_scoped_wired
from checks.e2e_verify_checks_out_pr_head.cli import cli as e2e_verify_checks_out_pr_head
from checks.e2e_verify_scope_wired.cli import cli as e2e_verify_scope_wired
from checks.e2e_verify_wired.cli import cli as e2e_verify_wired
from checks.gates_wired.cli import cli as gates_wired
from checks.github_helpers_wired.cli import cli as github_helpers_wired
from checks.hermetic_wired.cli import cli as hermetic_wired
from checks.integration_lint_new_arms_trip.cli import cli as integration_lint_new_arms_trip
from checks.isolation_red.cli import cli as isolation_red
from checks.isolation_wired.cli import cli as isolation_wired
from checks.mutation_gate.cli import cli as mutation_gate
from checks.mutation_package_root_wired.cli import cli as mutation_package_root_wired
from checks.mutation_wired.cli import cli as mutation_wired
from checks.packaging_build_wired.cli import cli as packaging_build_wired
from checks.packaging_package_root_red.cli import cli as packaging_package_root_red
from checks.packaging_package_root_wired.cli import cli as packaging_package_root_wired
from checks.packaging_red.cli import cli as packaging_red
from checks.python_mutation_clean.cli import cli as python_mutation_clean
from checks.red_path_hermetic_wired.cli import cli as red_path_hermetic_wired
from checks.rolling_release_wired.cli import cli as rolling_release_wired
from checks.rust_toolchain_wired.cli import cli as rust_toolchain_wired
from checks.static_gates_wired.cli import cli as static_gates_wired
from checks.suite_toolchain_pinned_wired.cli import cli as suite_toolchain_pinned_wired
from checks.uv_provisioning_wired.cli import cli as uv_provisioning_wired
from checks.verify_release.cli import cli as verify_release
from checks.verify_release_wired.cli import cli as verify_release_wired
from checks.wiring_detect_action.cli import cli as wiring_detect_action
from checks.wiring_e2e_default_on.cli import cli as wiring_e2e_default_on
from checks.wiring_packaging_default_on.cli import cli as wiring_packaging_default_on


@click.group()
def main() -> None:
    """Repo-only self-test wiring/assertion checks (#321)."""


main.add_command(below_floor, name="below-floor")
main.add_command(build_command_wired, name="build-command-wired")
main.add_command(build_hermetic_cli, name="build-hermetic-cli")
main.add_command(colocated_rust_red, name="colocated-rust-red")
main.add_command(colocated_rust_wired, name="colocated-rust-wired")
main.add_command(coverage_package_root_wired, name="coverage-package-root-wired")
main.add_command(coverage_rust_red, name="coverage-rust-red")
main.add_command(coverage_rust_wired, name="coverage-rust-wired")
main.add_command(detect_package_root_py, name="detect-package-root-py")
main.add_command(detect_package_root_ts, name="detect-package-root-ts")
main.add_command(detect_routes_python, name="detect-routes-python")
main.add_command(diff_scoped_wired, name="diff-scoped-wired")
main.add_command(e2e_verify_checks_out_pr_head, name="e2e-verify-checks-out-pr-head")
main.add_command(e2e_verify_scope_wired, name="e2e-verify-scope-wired")
main.add_command(e2e_verify_wired, name="e2e-verify-wired")
main.add_command(gates_wired, name="gates-wired")
main.add_command(github_helpers_wired, name="github-helpers-wired")
main.add_command(hermetic_wired, name="hermetic-wired")
main.add_command(integration_lint_new_arms_trip, name="integration-lint-new-arms-trip")
main.add_command(isolation_red, name="isolation-red")
main.add_command(isolation_wired, name="isolation-wired")
main.add_command(mutation_gate, name="mutation-gate")
main.add_command(mutation_package_root_wired, name="mutation-package-root-wired")
main.add_command(mutation_wired, name="mutation-wired")
main.add_command(packaging_build_wired, name="packaging-build-wired")
main.add_command(packaging_package_root_red, name="packaging-package-root-red")
main.add_command(packaging_package_root_wired, name="packaging-package-root-wired")
main.add_command(packaging_red, name="packaging-red")
main.add_command(python_mutation_clean, name="python-mutation-clean")
main.add_command(red_path_hermetic_wired, name="red-path-hermetic-wired")
main.add_command(rolling_release_wired, name="rolling-release-wired")
main.add_command(rust_toolchain_wired, name="rust-toolchain-wired")
main.add_command(static_gates_wired, name="static-gates-wired")
main.add_command(suite_toolchain_pinned_wired, name="suite-toolchain-pinned-wired")
main.add_command(uv_provisioning_wired, name="uv-provisioning-wired")
main.add_command(verify_release, name="verify-release")
main.add_command(verify_release_wired, name="verify-release-wired")
main.add_command(wiring_detect_action, name="wiring-detect-action")
main.add_command(wiring_e2e_default_on, name="wiring-e2e-default-on")
main.add_command(wiring_packaging_default_on, name="wiring-packaging-default-on")
