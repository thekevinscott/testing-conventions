"""Colocated unit tests for the verify-release command group (isolation — no `CliRunner`).

The decisions and parses are covered in `checks/utils/verify_release_test.py`; the subcommands
here are thin glue over that module (and exempted from coverage/mutation, like
`build_hermetic_cli.cli`). This pins the group's declared structure — its subcommand names and
their arguments — without a `CliRunner` collaborator or a real git/gh.
"""
from checks.verify_release.cli import cli


def test_registers_the_three_verify_release_subcommands():
    assert set(cli.commands) == {"resolve-version", "check-layout", "dispatch-and-wait"}


def test_resolve_version_and_check_layout_each_take_a_single_sha_argument():
    for name in ("resolve-version", "check-layout"):
        (argument,) = cli.commands[name].params
        assert argument.name == "sha"


def test_dispatch_and_wait_takes_sha_version_and_variadic_workflows():
    sha, version, workflows = cli.commands["dispatch-and-wait"].params
    assert sha.name == "sha"
    assert version.name == "version"
    assert workflows.name == "workflows"
    assert workflows.nargs == -1
