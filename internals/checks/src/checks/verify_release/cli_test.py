"""Colocated unit tests for the verify-release command group (isolation — no `CliRunner`, no git/gh).

The parses and timing decisions are covered beside `checks/utils/verify_release/`; each callback's
own raise-or-echo branch is driven here through `.callback()` with the imported collaborators
patched by string target.
"""
from checks.verify_release.cli import cli


def _record(seen, name, answer):
    def call(*args):
        seen[name] = args
        return answer

    return call


def test_registers_the_three_verify_release_subcommands():
    assert set(cli.commands) == {"resolve-version", "check-layout", "dispatch-and-wait"}


def test_resolve_version_and_check_layout_each_take_a_single_sha_argument():
    for name in ("resolve-version", "check-layout"):
        (argument,) = cli.commands[name].params
        assert argument.name == "sha"


def test_resolve_version_echoes_the_version_resolved_at_the_sha(monkeypatch, capsys):
    seen = {}
    monkeypatch.setattr(
        "checks.verify_release.cli.resolve_published_version", _record(seen, "resolve", "0.0.67")
    )
    cli.commands["resolve-version"].callback(sha="thesha")
    assert seen["resolve"] == ("thesha",)
    assert capsys.readouterr().out == "0.0.67\n"


def test_check_layout_echoes_the_ok_message_when_no_path_is_absent(monkeypatch, capsys):
    seen = {}
    monkeypatch.setattr("checks.verify_release.cli.find_missing_layout_paths", _record(seen, "check", []))
    monkeypatch.setattr("checks.verify_release.cli.layout_ok", _record(seen, "ok", "the fetch layout holds at thesha"))
    monkeypatch.setattr("checks.verify_release.cli.layout_error", _record(seen, "error", "unreachable"))
    cli.commands["check-layout"].callback(sha="thesha")
    assert seen["check"] == ("thesha",)
    assert seen["ok"] == ("thesha",)
    assert capsys.readouterr().out == "the fetch layout holds at thesha\n"


def test_check_layout_raises_the_error_message_when_a_path_is_absent(monkeypatch):
    seen = {}
    monkeypatch.setattr(
        "checks.verify_release.cli.find_missing_layout_paths",
        _record(seen, "check", ["internals/detect/src/detect.py"]),
    )
    monkeypatch.setattr("checks.verify_release.cli.layout_ok", _record(seen, "ok", "unreachable"))
    monkeypatch.setattr(
        "checks.verify_release.cli.layout_error", _record(seen, "error", "detect.py is absent at thesha")
    )
    try:
        cli.commands["check-layout"].callback(sha="thesha")
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert error.message == "detect.py is absent at thesha"
    else:
        raise AssertionError("an absent path must raise")
    assert seen["error"] == ("thesha", ["internals/detect/src/detect.py"])
