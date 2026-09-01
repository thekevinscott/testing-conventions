"""Colocated unit tests for the verify-release command group (isolation — no `CliRunner`, no git/gh).

The parses and timing decisions are covered beside `checks/utils/verify_release.py`; each callback's
own raise-or-echo branch is driven here through `.callback()` with `vr` patched by string target.
"""
from types import SimpleNamespace

from checks.verify_release.cli import cli


def _patch_vr(monkeypatch, **answers):
    """Stand a recording namespace in for the verify_release module; return the args each member saw."""
    seen = {}

    def member(name, answer):
        def call(*args):
            seen[name] = args
            return answer

        return call

    namespace = SimpleNamespace(**{name: member(name, answer) for name, answer in answers.items()})
    monkeypatch.setattr("checks.verify_release.cli.vr", namespace)
    return seen


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


def test_resolve_version_echoes_the_version_resolved_at_the_sha(monkeypatch, capsys):
    seen = _patch_vr(monkeypatch, resolve_version="0.0.67")
    cli.commands["resolve-version"].callback(sha="thesha")
    assert seen["resolve_version"] == ("thesha",)
    assert capsys.readouterr().out == "0.0.67\n"


def test_check_layout_echoes_the_ok_message_when_no_path_is_absent(monkeypatch, capsys):
    seen = _patch_vr(
        monkeypatch,
        check_layout=[],
        layout_ok="the fetch layout holds at thesha",
        layout_error="unreachable",
    )
    cli.commands["check-layout"].callback(sha="thesha")
    assert seen["check_layout"] == ("thesha",)
    assert seen["layout_ok"] == ("thesha",)
    assert capsys.readouterr().out == "the fetch layout holds at thesha\n"


def test_check_layout_raises_the_error_message_when_a_path_is_absent(monkeypatch):
    seen = _patch_vr(
        monkeypatch,
        check_layout=["internals/detect/src/detect.py"],
        layout_ok="unreachable",
        layout_error="detect.py is absent at thesha",
    )
    try:
        cli.commands["check-layout"].callback(sha="thesha")
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert error.message == "detect.py is absent at thesha"
    else:
        raise AssertionError("an absent path must raise")
    assert seen["layout_error"] == ("thesha", ["internals/detect/src/detect.py"])


def test_dispatch_and_wait_echoes_the_ok_message_when_every_suite_passed(monkeypatch, capsys):
    conclusions = {"dogfood.yml": "success"}
    seen = _patch_vr(
        monkeypatch,
        verify_suites=conclusions,
        failed_suites=[],
        verification_ok="both suites passed at thesha",
        verification_error="unreachable",
    )
    cli.commands["dispatch-and-wait"].callback(
        sha="thesha", version="0.0.67", workflows=("selftest.yml", "dogfood.yml")
    )
    assert seen["verify_suites"] == ("thesha", "0.0.67", ["selftest.yml", "dogfood.yml"])
    assert seen["failed_suites"] == (conclusions,)
    assert seen["verification_ok"] == ("thesha", conclusions)
    assert capsys.readouterr().out == "both suites passed at thesha\n"


def test_dispatch_and_wait_raises_the_error_message_when_a_suite_failed(monkeypatch):
    seen = _patch_vr(
        monkeypatch,
        verify_suites={"dogfood.yml": "failure"},
        failed_suites=["dogfood.yml"],
        verification_ok="unreachable",
        verification_error="dogfood.yml failed at thesha",
    )
    try:
        cli.commands["dispatch-and-wait"].callback(sha="thesha", version="0.0.67", workflows=("dogfood.yml",))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert error.message == "dogfood.yml failed at thesha"
    else:
        raise AssertionError("a failed suite must raise")
    assert seen["verification_error"] == ("thesha", ["dogfood.yml"])
